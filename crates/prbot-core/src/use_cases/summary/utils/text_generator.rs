use prbot_models::{ChecksStatus, MergeStrategy, QaStatus};

use crate::{
    use_cases::status::{PullRequestStatus, StatusMessageGenerator},
    Result,
};

/// Summary text generator.
pub struct SummaryTextGenerator;

impl SummaryTextGenerator {
    /// Generates status comment.
    pub fn generate(pull_request_status: &PullRequestStatus) -> Result<String> {
        Ok(format!(
            "_This is an auto-generated message summarizing this pull request._\n\
            \n\
            {rules_section}\n\
            \n\
            {checks_section}\n\
            \n\
            {config_section}\n\
            \n\
            {footer}",
            rules_section = Self::generate_status_comment_rule_section(
                pull_request_status.valid_pr_title,
                &pull_request_status.pull_request_title_regex,
                pull_request_status.merge_strategy,
                &pull_request_status.rule_names()
            ),
            checks_section = Self::generate_status_comment_checks_section(pull_request_status),
            config_section =
                Self::generate_status_comment_config_section(pull_request_status.automerge),
            footer = Self::generate_status_comment_footer(pull_request_status)?
        ))
    }

    fn generate_status_comment_rule_section(
        valid_pull_request_title: bool,
        pull_request_title_regex: &str,
        strategy: MergeStrategy,
        rule_names: &Option<String>,
    ) -> String {
        let validation_regex = if pull_request_title_regex.is_empty() {
            "None".to_owned()
        } else {
            format!("`{}`", pull_request_title_regex)
        };

        let title_validation_status = if valid_pull_request_title {
            "_valid!_ :heavy_check_mark:"
        } else {
            "_invalid!_ :x:"
        };

        let pr_rules = match rule_names {
            None => "None".to_owned(),
            Some(rules) => format!("**{}**", rules),
        };

        format!(
            ":pencil: &mdash; **Rules**\n\
            \n\
            > - :speech_balloon: **Title validation**: {status}\n\
            >   - _Rule:_ {rule}\n\
            > - :twisted_rightwards_arrows: **Merge strategy**: _{strategy}_\n\
            > - :straight_ruler: **Pull request rules**: _{pr_rules}_\n",
            status = title_validation_status,
            rule = validation_regex,
            strategy = strategy,
            pr_rules = pr_rules
        )
    }

    fn generate_status_comment_checks_section(pull_request_status: &PullRequestStatus) -> String {
        let checks_message = match pull_request_status.checks_status {
            ChecksStatus::Pass => "_passed!_ :heavy_check_mark:",
            ChecksStatus::Waiting => "_running..._ :clock2:",
            ChecksStatus::Fail => "_failed._ :x:",
            ChecksStatus::Skipped => "_skipped._ :heavy_check_mark:",
        };

        let qa_message = match pull_request_status.qa_status {
            QaStatus::Pass => "_passed!_ :heavy_check_mark:",
            QaStatus::Waiting => "_waiting..._ :clock2:",
            QaStatus::Fail => "_failed._ :x:",
            QaStatus::Skipped => "_skipped._ :heavy_check_mark:",
        };

        let lock_message = if pull_request_status.locked {
            "Yes :x:"
        } else {
            "No :heavy_check_mark:"
        };

        let wip_message = if pull_request_status.wip {
            "Yes :x:"
        } else {
            "No :heavy_check_mark:"
        };

        let mergeable_message = if pull_request_status.mergeable || pull_request_status.merged {
            "Yes :heavy_check_mark:"
        } else {
            "No :x:"
        };

        let code_review_section = if pull_request_status.changes_required() {
            format!(
                "_waiting on change requests..._ ({}) :clock2:",
                pull_request_status.changes_required_reviewers.join(", ")
            )
        } else if pull_request_status.missing_required_reviews() {
            format!(
                "_waiting on mandatory reviews..._ ({}) :clock2:",
                pull_request_status.missing_required_reviewers.join(", ")
            )
        } else if pull_request_status.missing_reviews() {
            format!(
                "_waiting..._ ({} given / {} required) :clock2:",
                pull_request_status.approved_reviewers.len(),
                pull_request_status.needed_reviewers_count
            )
        } else {
            format!(
                "_passed! ({} given / {} required)_ :heavy_check_mark:",
                pull_request_status.approved_reviewers.len(),
                pull_request_status.needed_reviewers_count
            )
        };

        format!(
            ":speech_balloon: &mdash; **Status comment**\n\
            \n\
            > - :construction: **WIP?** {wip_message}\n\
            > - :checkered_flag: **Checks**: {checks_message}\n\
            > - :mag: **Code reviews**: {reviews_message}\n\
            > - :test_tube: **QA**: {qa_message}\n\
            > - :lock: **Locked?**: {lock_message}\n\
            > - :twisted_rightwards_arrows: **Mergeable?**: {mergeable_message}",
            wip_message = wip_message,
            checks_message = checks_message,
            reviews_message = code_review_section,
            qa_message = qa_message,
            lock_message = lock_message,
            mergeable_message = mergeable_message,
        )
    }

    fn generate_status_comment_config_section(automerge: bool) -> String {
        let automerge_message = if automerge {
            ":heavy_check_mark:"
        } else {
            ":x:"
        };

        format!(
            ":gear: &mdash; **Configuration**\n\
            \n\
            > - :twisted_rightwards_arrows: **Automerge**: {automerge}",
            automerge = automerge_message
        )
    }

    fn generate_status_comment_footer(pull_request_status: &PullRequestStatus) -> Result<String> {
        let status_message = StatusMessageGenerator::default().generate(pull_request_status)?;

        Ok(format!(
            ":scroll: &mdash; **Current status**\n\
            \n\
            > _{status_state:?}: {status_msg}_\n\
            \n\
            [_See checks output by clicking this link :triangular_flag_on_post:_]({checks_url})",
            checks_url = pull_request_status.checks_url,
            status_state = status_message.state,
            status_msg = status_message.message
        ))
    }
}
