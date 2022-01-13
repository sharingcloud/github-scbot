use github_scbot_database::models::{IDatabaseAdapter, PullRequestModel, RepositoryModel};
use juniper::{EmptySubscription, FieldResult, RootNode};

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};

use crate::server::AppContext;

impl juniper::Context for AppContext {}

#[derive(GraphQLObject)]
#[graphql(description = "GitHub Repository")]
struct Repository {
    name: String,
    owner: String,
    pr_title_validation_regex: String,
    manual_interaction: bool,
    default_strategy: String,
    default_needed_reviewers_count: i32,
    default_automerge: bool,
    default_enable_qa: bool,
    default_enable_checks: bool,
}

impl From<github_scbot_database::models::RepositoryModel> for Repository {
    fn from(m: github_scbot_database::models::RepositoryModel) -> Self {
        Self {
            name: m.name().to_string(),
            owner: m.owner().to_string(),
            pr_title_validation_regex: m.pr_title_validation_regex().to_string(),
            manual_interaction: m.manual_interaction(),
            default_strategy: m.default_merge_strategy().to_string(),
            default_needed_reviewers_count: m.default_needed_reviewers_count(),
            default_automerge: m.default_automerge(),
            default_enable_qa: m.default_enable_qa(),
            default_enable_checks: m.default_enable_checks(),
        }
    }
}

#[derive(GraphQLObject)]
#[graphql(description = "GitHub Pull Request")]
struct PullRequest {
    repository_id: i32,
    number: i32,
    creator: String,
    name: String,
    base_branch: String,
    head_branch: String,
    step: Option<String>,
    check_status: String,
    qa_status: String,
    needed_reviewers_count: i32,
    status_comment_id: i32,
    automerge: bool,
    wip: bool,
    locked: bool,
    merged: bool,
    closed: bool,
    strategy_override: Option<String>,
}

impl From<github_scbot_database::models::PullRequestModel> for PullRequest {
    fn from(m: github_scbot_database::models::PullRequestModel) -> Self {
        Self {
            repository_id: m.repository_id(),
            number: m.number() as i32,
            creator: m.creator().to_owned(),
            name: m.name().to_owned(),
            base_branch: m.base_branch().to_owned(),
            head_branch: m.head_branch().to_owned(),
            step: m.step().map(|x| x.to_string()),
            check_status: m.check_status().to_string(),
            qa_status: m.qa_status().to_string(),
            needed_reviewers_count: m.needed_reviewers_count(),
            status_comment_id: m.status_comment_id() as i32,
            automerge: m.automerge(),
            wip: m.wip(),
            locked: m.locked(),
            merged: m.merged(),
            closed: m.closed(),
            strategy_override: m.strategy_override().map(|x| x.to_string()),
        }
    }
}

#[derive(GraphQLObject)]
#[graphql(description = "GitHub Review")]
struct Review {
    pull_request: PullRequest,
    username: String,
    state: String,
    required: bool,
    valid: bool,
}

#[derive(GraphQLObject)]
#[graphql(description = "Merge Rule")]
struct MergeRule {
    repository_id: i32,
    base_branch: String,
    head_branch: String,
    strategy: String,
}

impl From<github_scbot_database::models::MergeRuleModel> for MergeRule {
    fn from(m: github_scbot_database::models::MergeRuleModel) -> Self {
        Self {
            repository_id: m.repository_id(),
            base_branch: m.base_branch().to_string(),
            head_branch: m.head_branch().to_string(),
            strategy: m.strategy().to_string(),
        }
    }
}

#[derive(GraphQLObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct Human {
    id: String,
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct NewHuman {
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

pub struct QueryRoot;

#[juniper::graphql_object(Context = AppContext)]
impl QueryRoot {
    async fn repositories(context: &AppContext) -> FieldResult<Vec<Repository>> {
        let repositories = context.db_adapter.repository().list().await?;
        Ok(repositories.into_iter().map(Into::into).collect())
    }

    async fn pull_requests(
        context: &AppContext,
        repository_path: String,
    ) -> FieldResult<Vec<PullRequest>> {
        let pull_requests = context
            .db_adapter
            .pull_request()
            .list_from_repository_path(&repository_path)
            .await?;
        Ok(pull_requests.into_iter().map(Into::into).collect())
    }

    async fn merge_rules(
        context: &AppContext,
        repository_path: String,
    ) -> FieldResult<Vec<MergeRule>> {
        let repository =
            RepositoryModel::get_from_path(context.db_adapter.repository(), &repository_path)
                .await?;
        let merge_rules = context
            .db_adapter
            .merge_rule()
            .list_from_repository_id(repository.id())
            .await?;
        Ok(merge_rules.into_iter().map(Into::into).collect())
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = AppContext)]
impl MutationRoot {
    fn create_human(context: &AppContext, new_human: NewHuman) -> FieldResult<Human> {
        Ok(Human {
            id: "1234".to_owned(),
            name: new_human.name,
            appears_in: new_human.appears_in,
            home_planet: new_human.home_planet,
        })
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<AppContext>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}
