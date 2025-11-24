use entity::release;
use entity::sea_orm_active_enums::ReleaseType;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};
use utoipa::{IntoParams, ToSchema};

/// 发行版筛选器
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, ToSchema, IntoParams)]
#[schema(as = ReleaseFilter)]
pub struct ReleaseFilter {
    /// 发行类型集合
    #[serde_as(as = "Option<OneOrMany<_, serde_with::formats::PreferOne>>")]
    #[serde(default, rename = "release_type", alias = "release_type[]")]
    pub release_types: Option<Vec<ReleaseType>>,
}

impl ReleaseFilter {
    /// 将过滤器转换为 `Select<release::Entity>` 查询
    pub fn into_select(self) -> Select<release::Entity> {
        let mut select = release::Entity::find();

        // 发行类型过滤
        if let Some(release_types) = &self.release_types {
            select = select.filter(release::Column::ReleaseType.is_in(release_types.clone()));
        }

        select
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{QuerySelect, QueryTrait};

    use super::ReleaseFilter;
    use entity::sea_orm_active_enums::ReleaseType;

    #[test]
    fn release_type_filter_into_select_query_sql() {
        let filter = ReleaseFilter {
            release_types: Some(vec![ReleaseType::Album, ReleaseType::Single]),
        };

        // 仅选择常量以简化断言
        let query: sea_orm::Select<entity::release::Entity> =
            filter.into_select().select_only().expr(1);
        let sql = query.build(sea_orm::DatabaseBackend::Postgres).to_string();
        println!("{}", sql);
        let expected = "SELECT 1 FROM \"release\" WHERE \"release\".\"release_type\" IN (CAST('Album' AS \"ReleaseType\"), CAST('Single' AS \"ReleaseType\"))";
        // 断言包含核心的筛选片段
        assert!(sql.contains(expected));
    }

    #[test]
    fn empty_filters_into_select_query_sql() {
        let filter = ReleaseFilter {
            release_types: None,
        };

        // 仅选择常量以简化断言
        let query: sea_orm::Select<entity::release::Entity> =
            filter.into_select().select_only().expr(1);
        let sql = query.build(sea_orm::DatabaseBackend::Postgres).to_string();
        println!("{}", sql);
        let expected = "SELECT 1 FROM \"release\"";
        // 断言不包含任何筛选条件
        assert!(sql.eq(expected));
    }
}