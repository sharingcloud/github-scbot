export interface Repository {
	id: number,
	owner: string,
	name: string,
	manual_interaction: boolean,
	pr_title_validation_regex: string,
	default_strategy: MergeStrategy,
	default_needed_reviewers_count: number,
	default_automerge: boolean,
	default_enable_qa: boolean,
	default_enable_checks: boolean
}

export interface PullRequest {
	id: number,
	repository_id: number,
	number: number,
	qa_status: QaStatus,
	needed_reviewers_count: number,
	status_comment_id: number,
	checks_enabled: boolean,
	automerge: boolean,
	locked: boolean,
	strategy_override: MergeStrategy | null
}

export interface ExtendedRepository {
	repository: Repository,
	pull_requests: PullRequest[],
	merge_rules: MergeRule[],
	pull_request_rules: PullRequestRule[]
}

export interface Account {
	username: string,
	is_admin: boolean
}

export interface ExternalAccount {
	username: string,
	public_key: string,
	private_key: string
}

export interface ExtendedExternalAccount {
	external_account: ExternalAccount,
	rights: ExternalAccountRight[]
}

export interface ExternalAccountRight {
	username: string,
	repository_id: number
}

export interface MergeRule {
	repository_id: number,
	base_branch: RuleBranch,
	head_branch: RuleBranch,
	strategy: MergeStrategy
}

export enum MergeStrategy {
	Merge = "merge",
	Squash = "squash",
	Rebase = "rebase"
}

export enum QaStatus {
	Waiting = "waiting",
	Skipped = "skipped",
	Pass = "pass",
	Fail = "fail"
}

export interface RuleBranchWildcard {
	kind: "wildcard"
}

export interface RuleBranchNamed {
	kind: "named",
	name: string
}

export type RuleBranch = RuleBranchWildcard | RuleBranchNamed;

type RuleActionSetAutomerge = { ["set_automerge"]: boolean };
type RuleActionSetQaEnabled = { ["set_qa_enabled"]: boolean };
type RuleActionSetChecksEnabled = { ["set_checks_enabled"]: boolean };
type RuleActionSetNeededReviewers = { ["set_needed_reviewers"]: number };

export type RuleAction = RuleActionSetAutomerge | RuleActionSetQaEnabled | RuleActionSetChecksEnabled | RuleActionSetNeededReviewers;

type RuleConditionBaseBranch = { ["base_branch"]: RuleBranch };
type RuleConditionHeadBranch = { ["head_branch"]: RuleBranch };
type RuleConditionAuthor = { ["author"]: string };

export type RuleCondition = RuleConditionBaseBranch | RuleConditionHeadBranch | RuleConditionAuthor;

export interface PullRequestRule {
	repository_id: number,
	name: string,
	conditions: RuleCondition[],
	actions: RuleAction[]
}
