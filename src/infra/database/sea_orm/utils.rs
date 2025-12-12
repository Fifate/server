use std::{env, str};

use entity::{user, user_role};
use sea_orm::ActiveValue::*;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, Iterable, PaginatorTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};

use crate::constant::ADMIN_USERNAME;
use crate::domain::auth::hash_password;
use crate::domain::model::UserRoleEnum;
use crate::shared::http::CorrectionSortField;

pub async fn correction_sorted_entity_ids(
    db: &impl ConnectionTrait,
    entity_type: entity::enums::EntityType,
    sort_field: CorrectionSortField,
    sort_direction: sea_orm::Order,
) -> Result<Vec<i32>, DbErr> {
    use entity::correction::Column;

    let models: Vec<entity::correction::Model> =
        entity::correction::Entity::find()
            .filter(Column::EntityType.eq(entity_type))
            .order_by(
                match sort_field {
                    CorrectionSortField::CreatedAt => Column::CreatedAt,
                    CorrectionSortField::HandledAt => Column::HandledAt,
                },
                sort_direction,
            )
            .all(db)
            .await?;

    Ok(models.into_iter().map(|m| m.entity_id).collect())
}

async fn username_in_use(
    username: &str,
    db: &impl ConnectionTrait,
) -> Result<bool, DbErr> {
    let user = user::Entity::find()
        .filter(user::Column::Name.eq(username))
        .count(db)
        .await?;

    Ok(user > 0)
}

pub async fn upsert_admin_acc(db: &DatabaseConnection) {
    let password = hash_password(
        &env::var("ADMIN_PASSWORD").expect("Env var ADMIN_PASSWORD is not set"),
    )
    .unwrap();

    async {
        let tx = db.begin().await?;

        if username_in_use(ADMIN_USERNAME, &tx).await? {
            user::Entity::update_many()
                .col_expr(user::Column::Password, Expr::value(password))
                .filter(user::Column::Name.eq(ADMIN_USERNAME))
                .exec(&tx)
                .await?;

            return Ok(());
        }

        let res = user::Entity::insert(user::ActiveModel {
            id: NotSet,
            name: Set(ADMIN_USERNAME.to_string()),
            password: Set(password),
            avatar_id: Set(None),
            profile_banner_id: Set(None),
            last_login: Set(chrono::Local::now().into()),
            bio: Set(None),
        })
        .on_conflict(
            OnConflict::column(user::Column::Name)
                .update_columns(user::Column::iter())
                .to_owned(),
        )
        .exec_with_returning(&tx)
        .await?;

        user_role::Entity::insert(
            user_role::Model {
                user_id: res.id,
                role_id: UserRoleEnum::Admin.into(),
            }
            .into_active_model(),
        )
        .on_conflict_do_nothing()
        .exec(&tx)
        .await?;

        tx.commit().await
    }
    .await
    .expect("Failed to upsert admin account");
}

pub fn sort_by_id_list<T>(
    mut items: Vec<T>,
    id_order: &[i32],
    get_id: impl Fn(&T) -> i32,
) -> Vec<T> {
    use std::collections::HashMap;

    let id_to_index: HashMap<i32, usize> = id_order
        .iter()
        .enumerate()
        .map(|(index, &id)| (id, index))
        .collect();

    items.sort_by_key(|item| {
        id_to_index
            .get(&get_id(item))
            .copied()
            .unwrap_or(usize::MAX)
    });

    items
}
