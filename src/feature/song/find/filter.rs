use entity::{song, song_language};
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Select,
};
use sea_query::Expr;
use serde::Deserialize;
use serde_with::{DisplayFromStr, OneOrMany, serde_as};
use utoipa::{IntoParams, ToSchema};

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
}

impl SongFilter {
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

    use super::SongFilter;

    #[test]
    fn language_filter_into_select_query_sql() {
        let filter = SongFilter {
            exclusion: None,
            language_ids: Some(vec![1, 2]),
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
}
