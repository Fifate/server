use entity::{song, song_artist};
use sea_orm::{ColumnTrait, QueryFilter, Select, EntityTrait, QuerySelect, QueryTrait};
use sea_query::{Expr};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use serde_with::{serde_as, DisplayFromStr, OneOrMany};

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
}

impl SongFilter {
    /// 将过滤器转换为 `Select<song::Entity>` 查询
    pub fn into_select(self) -> Select<song::Entity> {
        let mut select = song::Entity::find();

        // 排除部分
        if let Some(exclusion) = &self.exclusion {
            if !exclusion.is_empty() {
                select = select.filter(song::Column::Id.is_not_in(exclusion.clone()));
            }
        }

        // 艺术家过滤：通过 EXISTS 子查询匹配 song_artist 关系，避免联接重复
        if let Some(artist_ids) = &self.artist_ids {
            if !artist_ids.is_empty() {
                let artist_ids = artist_ids.clone();
                let subquery = song_artist::Entity::find()
                    .select_only()
                    .expr(1)
                    .filter(Expr::eq(
                        Expr::col((song_artist::Entity, song_artist::Column::SongId)),
                        Expr::col((song::Entity, song::Column::Id)),
                    ))
                    .filter(song_artist::Column::ArtistId.is_in(artist_ids));

                select = select.filter(Expr::exists(subquery.as_query().clone()));
            }
        }

        select
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{QueryTrait, QuerySelect};
    use super::SongFilter;

    #[test]
    fn artist_filter_into_select_query_sql() {
        let filter = SongFilter {
            artist_ids: Some(vec![1,2,3]),
            exclusion: None,
        };

        // 仅选择常量以简化断言
        let query: sea_orm::Select<entity::song::Entity> = filter.into_select().select_only().expr(1);
        let sql = query.build(sea_orm::DatabaseBackend::Postgres).to_string();
        println!("{}", sql);
        let expected = "SELECT 1 FROM \"song\" WHERE EXISTS(SELECT 1 FROM \"song_artist\" WHERE \"song_artist\".\"song_id\" = \"song\".\"id\" AND \"song_artist\".\"artist_id\" IN (1, 2, 3))";
        // 断言包含核心的连接与筛选片段（避免因列顺序导致的不稳定）
        assert!(sql.eq(expected));

    }
}