use entity::{song, song_language};
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Select,
};
use sea_query::Expr;
use serde::Deserialize;
use serde_with::{DisplayFromStr, OneOrMany, serde_as};
use utoipa::{IntoParams, ToSchema};

/// 排序字段枚举
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    /// 按创建时间排序
    CreatedAt,
    /// 按处理时间排序
    HandledAt,
}

/// 排序方向枚举
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    /// 升序排序
    Asc,
    /// 降序排序
    Desc,
}

/// 可扩展的歌曲筛选器
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, ToSchema, IntoParams)]
#[schema(as = SongFilter)]
pub struct SongFilter {
    /// 排除的歌曲ID
    #[serde(default, alias = "exclusion[]")]
    #[serde_as(as = "Option<OneOrMany<DisplayFromStr>>")]
    pub exclusion: Option<Vec<i32>>,

    /// 语言ID集合，匹配歌曲的语言
    #[serde(default, rename = "language_id", alias = "language_id[]")]
    #[serde_as(as = "Option<OneOrMany<DisplayFromStr>>")]
    pub language_ids: Option<Vec<i32>>,

    /// 排序字段
    #[serde(default)]
    pub sort_field: Option<SortField>,

    /// 排序方向
    #[serde(default)]
    pub sort_direction: Option<SortDirection>,
}

impl SongFilter {
    pub fn with_sort_defaults(mut self) -> Self {
        if self.sort_field.is_none() {
            self.sort_field = Some(SortField::CreatedAt);
        }
        if self.sort_direction.is_none() {
            self.sort_direction = Some(SortDirection::Desc);
        }
        self
    }
    /// 将过滤器转换为 `Select<song::Entity>` 查询
    pub fn into_select(self) -> Select<song::Entity> {
        let mut select = song::Entity::find();

        // 排除部分
        if let Some(exclusion) = &self.exclusion {
            select =
                select.filter(song::Column::Id.is_not_in(exclusion.clone()));
        }

        // 语言过滤：通过 EXISTS 子查询匹配 song_language 关系
        if let Some(language_ids) = &self.language_ids {
            select = Self::apply_language_filter(select, language_ids.clone());
        }

        select
    }

    /// 应用语言过滤条件
    fn apply_language_filter(
        select: Select<song::Entity>,
        language_ids: Vec<i32>,
    ) -> Select<song::Entity> {
        let subquery = song_language::Entity::find()
            .select_only()
            .expr(1)
            .filter(Expr::eq(
                Expr::col((
                    song_language::Entity,
                    song_language::Column::SongId,
                )),
                Expr::col((song::Entity, song::Column::Id)),
            ))
            .filter(song_language::Column::LanguageId.is_in(language_ids));

        select.filter(Expr::exists(subquery.as_query().clone()))
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{QuerySelect, QueryTrait};

    use crate::feature::song::find::filter::SortField;
    use crate::feature::song::find::filter::SortDirection;

    use super::SongFilter;

    #[test]
    fn language_filter_into_select_query_sql() {
        let filter = SongFilter {
            exclusion: None,
            language_ids: Some(vec![1, 2]),
            sort_field: None,
            sort_direction: None,
        };

        // 仅选择常量以简化断言
        let query: sea_orm::Select<entity::song::Entity> =
            filter.into_select().select_only().expr(1);
        let sql = query.build(sea_orm::DatabaseBackend::Postgres).to_string();
        println!("{}", sql);
        let expected = "SELECT 1 FROM \"song\" WHERE EXISTS(SELECT 1 FROM \"song_language\" WHERE \"song_language\".\"song_id\" = \"song\".\"id\" AND \"song_language\".\"language_id\" IN (1, 2))";
        // 断言包含核心的连接与筛选片段（避免因列顺序导致的不稳定）
        assert!(sql.eq(expected));
    }

    #[test]
    fn empty_filters_into_select_query_sql() {
        let filter = SongFilter {
            exclusion: None,
            language_ids: None,
            sort_field: None,
            sort_direction: None,
        };

        // 仅选择常量以简化断言
        let query: sea_orm::Select<entity::song::Entity> =
            filter.into_select().select_only().expr(1);
        let sql = query.build(sea_orm::DatabaseBackend::Postgres).to_string();
        println!("{}", sql);
        let expected = "SELECT 1 FROM \"song\"";
        // 断言包含核心的连接与筛选片段（避免因列顺序导致的不稳定）
        assert!(sql.eq(expected));
    }

    #[test]
    fn sort_parameters_serialization() {
        // 测试排序字段序列化
        let field_json = serde_json::to_string(&SortField::CreatedAt).unwrap();
        assert_eq!(field_json, "\"created_at\"");
        
        let field_json = serde_json::to_string(&SortField::HandledAt).unwrap();
        assert_eq!(field_json, "\"handled_at\"");

        // 测试排序方向序列化
        let direction_json = serde_json::to_string(&SortDirection::Asc).unwrap();
        assert_eq!(direction_json, "\"asc\"");
        
        let direction_json = serde_json::to_string(&SortDirection::Desc).unwrap();
        assert_eq!(direction_json, "\"desc\"");

        // 测试完整过滤器序列化
        let filter = SongFilter {
            exclusion: Some(vec![1, 2]),
            language_ids: Some(vec![3, 4]),
            sort_field: Some(SortField::CreatedAt),
            sort_direction: Some(SortDirection::Desc),
        };
        
        let filter_json = serde_json::to_value(&filter).unwrap();
        assert_eq!(filter_json["sort_field"], "created_at");
        assert_eq!(filter_json["sort_direction"], "desc");
    }

    #[test]
    fn sort_parameters_deserialization() {
        // 测试排序字段反序列化
        let field: SortField = serde_json::from_str("\"created_at\"").unwrap();
        assert!(matches!(field, SortField::CreatedAt));
        
        let field: SortField = serde_json::from_str("\"handled_at\"").unwrap();
        assert!(matches!(field, SortField::HandledAt));

        // 测试排序方向反序列化
        let direction: SortDirection = serde_json::from_str("\"asc\"").unwrap();
        assert!(matches!(direction, SortDirection::Asc));
        
        let direction: SortDirection = serde_json::from_str("\"desc\"").unwrap();
        assert!(matches!(direction, SortDirection::Desc));

        // 测试完整过滤器反序列化
        let filter_json = r#"{
            "exclusion": [1, 2],
            "language_ids": [3, 4],
            "sort_field": "handled_at",
            "sort_direction": "asc"
        }"#;
        
        let filter: SongFilter = serde_json::from_str(filter_json).unwrap();
        assert_eq!(filter.exclusion, Some(vec![1, 2]));
        assert_eq!(filter.language_ids, Some(vec![3, 4]));
        assert!(matches!(filter.sort_field, Some(SortField::HandledAt)));
        assert!(matches!(filter.sort_direction, Some(SortDirection::Asc)));
    }

    #[test]
    fn invalid_sort_field_deserialization_fails() {
        let json = r#"{"sort_field":"unknown"}"#;
        let result: Result<SongFilter, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_sort_direction_deserialization_fails() {
        let json = r#"{"sort_direction":"descending"}"#;
        let result: Result<SongFilter, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn sort_defaults_applied_when_missing_both() {
        let filter = SongFilter {
            exclusion: None,
            language_ids: None,
            sort_field: None,
            sort_direction: None,
        };
        let normalized = filter.with_sort_defaults();
        assert!(matches!(normalized.sort_field, Some(SortField::CreatedAt)));
        assert!(matches!(normalized.sort_direction, Some(SortDirection::Desc)));
    }

    #[test]
    fn sort_defaults_direction_only() {
        let filter = SongFilter {
            exclusion: None,
            language_ids: None,
            sort_field: None,
            sort_direction: Some(SortDirection::Asc),
        };
        let normalized = filter.with_sort_defaults();
        assert!(matches!(normalized.sort_field, Some(SortField::CreatedAt)));
        assert!(matches!(normalized.sort_direction, Some(SortDirection::Asc)));
    }

    #[test]
    fn sort_defaults_field_only() {
        let filter = SongFilter {
            exclusion: None,
            language_ids: None,
            sort_field: Some(SortField::HandledAt),
            sort_direction: None,
        };
        let normalized = filter.with_sort_defaults();
        assert!(matches!(normalized.sort_field, Some(SortField::HandledAt)));
        assert!(matches!(normalized.sort_direction, Some(SortDirection::Desc)));
    }

    #[test]
    fn sort_no_change_when_both_present() {
        let filter = SongFilter {
            exclusion: None,
            language_ids: None,
            sort_field: Some(SortField::CreatedAt),
            sort_direction: Some(SortDirection::Asc),
        };
        let normalized = filter.with_sort_defaults();
        assert!(matches!(normalized.sort_field, Some(SortField::CreatedAt)));
        assert!(matches!(normalized.sort_direction, Some(SortDirection::Asc)));
    }
}
