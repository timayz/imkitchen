use evento::Executor;

pub struct ToggleInput {
    pub name: String,
}

impl<E: Executor> super::Module<E> {
    /// Flip one ingredient between checked and unchecked. Not idempotent: a
    /// retried call flips it back; clients that may retry use
    /// [`set_checked`](Self::set_checked).
    pub async fn toggle(
        &self,
        input: ToggleInput,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let shopping = self.load_for_check(&request_by, &input.name).await?;
        let checked = !shopping.checked.contains(&input.name);

        self.write_checked(shopping, input.name, checked, request_by)
            .await
    }
}
