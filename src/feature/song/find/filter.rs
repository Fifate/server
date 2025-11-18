use entity::{song, song_artist, song_language};
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Select,
};
use sea_query::Expr;
use serde::Deserialize;
use serde_with::{DisplayFromStr, OneOrMany, serde_as};
use utoipa::{IntoParams, ToSchema};

/// 可扩展的歌曲筛选器
/// 目前先实现按艺术家进行筛选，后续可以扩展更多维度（风格、情感、语言、用户标签等）。
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, ToSchema, IntoParams)]
#[schema(as = SongFilter)]
pub struct SongFilter {
    /// 艺术家ID集合，匹配歌曲的参与艺术家
    #[serde(default, rename = "artist_id", alias = "artist_id[]")]
    #[serde_as(as = "Option<OneOrMany<DisplayFromStr>>")]
    pub artist_ids: Option<Vec<i32>>,

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

        // 艺术家过滤：通过 EXISTS 子查询匹配 song_artist 关系，避免联接重复
        if let Some(artist_ids) = &self.artist_ids {
            select = Self::apply_artist_filter(select, artist_ids.clone());
        }

        // 语言过滤：通过 EXISTS 子查询匹配 song_language 关系
        if let Some(language_ids) = &self.language_ids {
            select = Self::apply_language_filter(select, language_ids.clone());
        }

        select
    }

    /// 应用艺术家过滤条件
    fn apply_artist_filter(
        select: Select<song::Entity>,
        artist_ids: Vec<i32>,
    ) -> Select<song::Entity> {
        Self::apply_relation_filter::<song_artist::Entity, song_artist::Column>(
            select,
            artist_ids,
            song_artist::Column::SongId,
            song_artist::Column::ArtistId,
        )
    }

    /// 应用语言过滤条件
    fn apply_language_filter(
        select: Select<song::Entity>,
        language_ids: Vec<i32>,
    ) -> Select<song::Entity> {
        Self::apply_relation_filter::<
            song_language::Entity,
            song_language::Column,
        >(
            select,
            language_ids,
            song_language::Column::SongId,
            song_language::Column::LanguageId,
        )
    }

    /// 通用关系过滤函数
    fn apply_relation_filter<E, C>(
        select: Select<song::Entity>,
        ids: Vec<i32>,
        song_id_column: C,
        target_id_column: C,
    ) -> Select<song::Entity>
    where
        E: EntityTrait,
        C: ColumnTrait + Clone,
    {
        let subquery = E::find()
            .select_only()
            .expr(1)
            .filter(Expr::eq(
                Expr::col((E::default(), song_id_column.clone())),
                Expr::col((song::Entity, song::Column::Id)),
            ))
            .filter(target_id_column.is_in(ids));

        select.filter(Expr::exists(subquery.as_query().clone()))
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{QuerySelect, QueryTrait};

    use super::SongFilter;

    #[test]
    fn artist_filter_into_select_query_sql() {
        let filter = SongFilter {
            artist_ids: Some(vec![1, 2, 3]),
            exclusion: None,
            language_ids: None,
        };

        // 仅选择常量以简化断言
        let query: sea_orm::Select<entity::song::Entity> =
            filter.into_select().select_only().expr(1);
        let sql = query.build(sea_orm::DatabaseBackend::Postgres).to_string();
        println!("{}", sql);
        let expected = "SELECT 1 FROM \"song\" WHERE EXISTS(SELECT 1 FROM \"song_artist\" WHERE \"song_artist\".\"song_id\" = \"song\".\"id\" AND \"song_artist\".\"artist_id\" IN (1, 2, 3))";
        // 断言包含核心的连接与筛选片段（避免因列顺序导致的不稳定）
        assert!(sql.eq(expected));
    }

    #[test]
    fn language_filter_into_select_query_sql() {
        let filter = SongFilter {
            artist_ids: None,
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
    fn combined_artist_and_language_filter_into_select_query_sql() {
        let filter = SongFilter {
            artist_ids: Some(vec![4, 5]),
            exclusion: None,
            language_ids: Some(vec![3]),
        };

        // 仅选择常量以简化断言
        let query: sea_orm::Select<entity::song::Entity> =
            filter.into_select().select_only().expr(1);
        let sql = query.build(sea_orm::DatabaseBackend::Postgres).to_string();
        println!("{}", sql);
        let expected = "SELECT 1 FROM \"song\" WHERE EXISTS(SELECT 1 FROM \"song_artist\" WHERE \"song_artist\".\"song_id\" = \"song\".\"id\" AND \"song_artist\".\"artist_id\" IN (4, 5)) AND EXISTS(SELECT 1 FROM \"song_language\" WHERE \"song_language\".\"song_id\" = \"song\".\"id\" AND \"song_language\".\"language_id\" IN (3))";
        // 断言包含核心的连接与筛选片段（避免因列顺序导致的不稳定）
        assert!(sql.eq(expected));
    }

    #[test]
    fn empty_filters_into_select_query_sql() {
        let filter = SongFilter {
            artist_ids: None,
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
