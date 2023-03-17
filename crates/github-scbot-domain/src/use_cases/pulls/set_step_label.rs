use github_scbot_domain_models::StepLabel;
use github_scbot_ghapi_interface::ApiService;

use crate::Result;

pub struct SetStepLabelUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
    pub label: Option<StepLabel>,
}

impl<'a> SetStepLabelUseCase<'a> {
    /// Add pull request step label in existing labels by returning a new vector.
    pub fn add_step_in_existing_labels(
        existing_labels: &[String],
        step: Option<StepLabel>,
    ) -> Vec<String> {
        let mut preserved_labels: Vec<String> = existing_labels
            .iter()
            .cloned()
            .filter(|x| StepLabel::try_from(&x[..]).is_err())
            .collect();

        if let Some(step) = step {
            preserved_labels.push(step.to_str().to_string());
        }

        preserved_labels
    }

    pub async fn run(&mut self) -> Result<()> {
        let existing_labels = self
            .api_service
            .issue_labels_list(self.repo_owner, self.repo_name, self.pr_number)
            .await?;
        let existing_labels = Self::add_step_in_existing_labels(&existing_labels, self.label);
        self.api_service
            .issue_labels_replace_all(
                self.repo_owner,
                self.repo_name,
                self.pr_number,
                &existing_labels,
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_ghapi_interface::MockApiService;

    use super::*;

    #[test]
    fn test_add_step_in_existing_labels() {
        // A step label remove existing step labels.
        assert_eq!(
            SetStepLabelUseCase::add_step_in_existing_labels(
                &[
                    "label1".into(),
                    "label2".into(),
                    StepLabel::AwaitingMerge.to_str().into()
                ],
                Some(StepLabel::AwaitingQa)
            ),
            vec![
                "label1".to_string(),
                "label2".to_string(),
                StepLabel::AwaitingQa.to_str().into()
            ]
        );

        // No step label remove existing step labels.
        assert_eq!(
            SetStepLabelUseCase::add_step_in_existing_labels(
                &[
                    "label1".into(),
                    "label2".into(),
                    StepLabel::AwaitingMerge.to_str().into()
                ],
                None
            ),
            vec!["label1".to_string(), "label2".to_string()]
        )
    }

    #[actix_rt::test]
    async fn test_set_step_label() -> Result<()> {
        let mut adapter = MockApiService::new();
        adapter
            .expect_issue_labels_list()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));
        adapter
            .expect_issue_labels_replace_all()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        SetStepLabelUseCase {
            api_service: &adapter,
            repo_owner: "owner",
            repo_name: "name",
            pr_number: 1,
            label: None,
        }
        .run()
        .await?;

        Ok(())
    }
}
