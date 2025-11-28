use bincode::{Decode, Encode};
use evento::{
    cursor::{Args, ReadResult},
    sql::Reader,
};
use imkitchen_db::table::{RecipeList, RecipeUserStat};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

use crate::{CuisineType, DietaryRestriction, Ingredient, Instruction, RecipeType, SortBy};

#[derive(Debug, Encode, Decode)]
pub struct RecipeQueryCursor {
    pub i: String,
    pub v: u64,
}

#[derive(Default, FromRow)]
pub struct RecipeRow {
    pub id: String,
    pub user_id: String,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub cuisine_type: sqlx::types::Text<CuisineType>,
    pub name: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: imkitchen_db::types::Bincode<Vec<Ingredient>>,
    pub instructions: imkitchen_db::types::Bincode<Vec<Instruction>>,
    pub dietary_restrictions: sqlx::types::Json<Vec<DietaryRestriction>>,
    pub accepts_accompaniment: bool,
    pub advance_prep: String,
    // pub is_shared: bool,
}

#[derive(Debug, Default, FromRow)]
pub struct RecipeListRow {
    pub id: String,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub cuisine_type: sqlx::types::Text<CuisineType>,
    pub name: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub dietary_restrictions: sqlx::types::Json<Vec<DietaryRestriction>>,
    pub accepts_accompaniment: bool,
    // pub is_shared: bool,
    pub created_at: u64,
}

impl evento::cursor::Cursor for RecipeListRow {
    type T = RecipeQueryCursor;

    fn serialize(&self) -> Self::T {
        Self::T {
            i: self.id.to_owned(),
            v: self.created_at,
        }
    }
}

impl evento::sql::Bind for RecipeListRow {
    type T = RecipeList;
    type I = [Self::T; 2];
    type V = [Expr; 2];
    type Cursor = Self;

    fn columns() -> Self::I {
        [RecipeList::CreatedAt, RecipeList::Id]
    }

    fn values(
        cursor: <<Self as evento::sql::Bind>::Cursor as evento::cursor::Cursor>::T,
    ) -> Self::V {
        [cursor.v.into(), cursor.i.into()]
    }
}

#[derive(Clone)]
pub struct Query(pub sqlx::SqlitePool);

pub struct RecipesQuery {
    pub user_id: Option<String>,
    pub recipe_type: Option<RecipeType>,
    pub cuisine_type: Option<CuisineType>,
    pub is_shared: Option<bool>,
    pub sort_by: SortBy,
    pub args: Args,
}

impl Query {
    pub async fn filter(&self, query: RecipesQuery) -> anyhow::Result<ReadResult<RecipeListRow>> {
        let mut statment = sea_query::Query::select()
            .columns([
                RecipeList::Id,
                RecipeList::RecipeType,
                RecipeList::CuisineType,
                RecipeList::Name,
                RecipeList::Description,
                RecipeList::PrepTime,
                RecipeList::CookTime,
                RecipeList::DietaryRestrictions,
                RecipeList::AcceptsAccompaniment,
                RecipeList::IsShared,
                RecipeList::CreatedAt,
            ])
            .from(RecipeList::Table)
            .to_owned();

        if let Some(user_id) = query.user_id {
            statment.and_where(Expr::col(RecipeList::UserId).eq(user_id));
        }

        if let Some(recipe_type) = query.recipe_type {
            statment.and_where(Expr::col(RecipeList::RecipeType).eq(recipe_type.to_string()));
        }

        if let Some(cuisine_type) = query.cuisine_type {
            statment.and_where(Expr::col(RecipeList::CuisineType).eq(cuisine_type.to_string()));
        }

        if let Some(is_shared) = query.is_shared {
            statment.and_where(Expr::col(RecipeList::IsShared).eq(is_shared));
        }

        let mut reader = Reader::new(statment);

        if matches!(query.sort_by, SortBy::RecentlyAdded) {
            reader.desc();
        }

        Ok(reader
            .args(query.args)
            .execute::<_, RecipeListRow, _>(&self.0)
            .await?)
    }

    pub async fn find(&self, id: impl Into<String>) -> anyhow::Result<Option<RecipeRow>> {
        let statment = sea_query::Query::select()
            .columns([
                RecipeList::Id,
                RecipeList::UserId,
                RecipeList::RecipeType,
                RecipeList::CuisineType,
                RecipeList::Name,
                RecipeList::Description,
                RecipeList::PrepTime,
                RecipeList::CookTime,
                RecipeList::Ingredients,
                RecipeList::Instructions,
                RecipeList::DietaryRestrictions,
                RecipeList::AcceptsAccompaniment,
                RecipeList::AdvancePrep,
                RecipeList::IsShared,
                RecipeList::CreatedAt,
                RecipeList::UpdatedAt,
            ])
            .from(RecipeList::Table)
            .and_where(Expr::col(RecipeList::Id).eq(id.into()))
            .limit(1)
            .to_owned();

        let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, RecipeRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}

#[derive(Default, FromRow)]
pub struct UserStat {
    pub total: u32,
    pub favorite: u32,
    pub shared: u32,
    pub from_community: u32,
}

impl Query {
    pub async fn find_user_stat(
        &self,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<UserStat>> {
        let user_id = user_id.into();
        let statment = sea_query::Query::select()
            .columns([
                RecipeUserStat::Total,
                RecipeUserStat::Shared,
                RecipeUserStat::Favorite,
                RecipeUserStat::FromCommunity,
            ])
            .from(RecipeUserStat::Table)
            .and_where(Expr::col(RecipeUserStat::UserId).eq(user_id))
            .to_owned();

        let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, UserStat, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}
