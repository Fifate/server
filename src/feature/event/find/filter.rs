use chrono::NaiveDate;
use entity::event;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub use crate::shared::http::{
    CorrectionSortField, PaginationQuery, SortDirection,
};

#[derive(Clone, Debug, Default, Deserialize, ToSchema, IntoParams)]
#[schema(as = EventFilter)]
pub struct EventFilter {
    pub start_date_from: Option<NaiveDate>,

    pub start_date_to: Option<NaiveDate>,

    #[serde(default)]
    pub sort_field: Option<CorrectionSortField>,

    #[serde(default)]
    pub sort_direction: Option<SortDirection>,
}

impl EventFilter {
    pub const fn with_sort_defaults(mut self) -> Self {
        if self.sort_field.is_none() {
            self.sort_field = Some(CorrectionSortField::CreatedAt);
        }
        if self.sort_direction.is_none() {
            self.sort_direction = Some(SortDirection::Desc);
        }
        self
    }

    pub fn into_select(self) -> Select<event::Entity> {
        let mut select = event::Entity::find();

        if let Some(start_date_from) = self.start_date_from {
            select =
                select.filter(event::Column::StartDate.gte(start_date_from));
        }
        if let Some(start_date_to) = self.start_date_to {
            select = select.filter(event::Column::StartDate.lte(start_date_to));
        }

        select
    }
}
