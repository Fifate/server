use chrono::NaiveDate;
use entity::label;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub use crate::shared::http::{
    CorrectionSortField, PaginationQuery, SortDirection,
};

#[derive(Clone, Debug, Default, Deserialize, ToSchema, IntoParams)]
#[schema(as = LabelFilter)]
pub struct LabelFilter {
    pub founded_date_from: Option<NaiveDate>,

    pub founded_date_to: Option<NaiveDate>,

    pub is_dissolved: Option<bool>,

    #[serde(default)]
    pub sort_field: Option<CorrectionSortField>,

    #[serde(default)]
    pub sort_direction: Option<SortDirection>,
}

impl LabelFilter {
    pub const fn with_sort_defaults(mut self) -> Self {
        if self.sort_field.is_none() {
            self.sort_field = Some(CorrectionSortField::CreatedAt);
        }
        if self.sort_direction.is_none() {
            self.sort_direction = Some(SortDirection::Desc);
        }
        self
    }

    pub fn into_select(self) -> Select<label::Entity> {
        let mut select = label::Entity::find();

        if let Some(founded_date_from) = self.founded_date_from {
            select = select
                .filter(label::Column::FoundedDate.gte(founded_date_from));
        }
        if let Some(founded_date_to) = self.founded_date_to {
            select =
                select.filter(label::Column::FoundedDate.lte(founded_date_to));
        }

        if let Some(is_dissolved) = self.is_dissolved {
            if is_dissolved {
                select =
                    select.filter(label::Column::DissolvedDate.is_not_null());
            } else {
                select = select.filter(label::Column::DissolvedDate.is_null());
            }
        }

        select
    }
}
