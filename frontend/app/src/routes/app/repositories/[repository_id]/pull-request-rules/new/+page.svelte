<script lang="ts">
	import type { PageData } from './$types';
	import type { RuleAction, RuleCondition } from "$api/types";
	import { ApiClient } from "$api/client";
	import { goto } from "$app/navigation";
	import Form from "$components/base/Form.svelte";
	import Button from "$components/base/Button.svelte";
	import FieldSet from "$components/base/FieldSet.svelte";
	import TextInput from "$components/base/TextInput.svelte";
	import Select from "$components/base/Select.svelte";
	import NumberInput from "$components/base/NumberInput.svelte";
	import CheckBox from "$components/base/CheckBox.svelte";

	let name = "";
	let conditions: RuleCondition[] = [];
	let actions: RuleAction[] = [];

	let currentConditionKind = "base_branch";
	let currentConditionValue = "";
	let currentActionKind = "set_automerge";
	let currentActionValue: string | boolean | number | null = null;

	let previousActionKind = currentActionKind;

	export let data: PageData;

	$: {
		if (previousActionKind != currentActionKind) {
			previousActionKind = currentActionKind;
			currentActionValue = null;
		}
	}

	$: console.log({currentActionKind})
</script>

<h3>Create a new pull request rule</h3>
<br />

<Form
	on:submit={async (e) => {
		e.preventDefault();

		let client = new ApiClient();
		await client.repositories.withId(data.repositoryId).pullRequestRules.create({
			repository_id: data.repositoryId,
			name,
			conditions,
			actions
		});

		goto("/app/repositories");
	}}
>
	<FieldSet>
		<TextInput labelText="Rule name" bind:value={name} placeholder="Enter rule name..." />
	</FieldSet>

	<FieldSet labelText="Conditions">
		{#if conditions.length > 0}
			<table>
				<tbody>
					{#each conditions as condition}
						<tr>
							<td>{Object.keys(condition)[0]}</td>
							<td>{Object.values(condition)[0]}</td>
							<Button on:click={() => {
								const toRemove = conditions.findIndex(c => c == condition);
								conditions = [...conditions.slice(0, toRemove), ...conditions.slice(toRemove + 1)];
							}}>Remove</Button>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}

		<div class="add-condition">
			<Select
				labelText="Condition to add"
				bind:value={currentConditionKind}
				options={{
					"base_branch": "Base branch",
					"head_branch": "Head branch",
					"author": "Author"
				}}
			/>
			<TextInput labelText="Value" bind:value={currentConditionValue} />
			<Button disabled={currentConditionValue === ""} on:click={() => {
				conditions = [...conditions, {[currentConditionKind]: currentConditionValue}];
				currentConditionValue = "";
			}}>Add</Button>
		</div>
	</FieldSet>

	<FieldSet labelText="Actions">
		{#if actions.length > 0}
			<table>
				<tbody>
					{#each actions as action}
						<tr>
							<td>{Object.keys(action)[0]}</td>
							<td>{Object.values(action)[0]}</td>
							<Button size="small" on:click={() => {
								const toRemove = actions.findIndex(a => a == action);
								actions = [...actions.slice(0, toRemove), ...actions.slice(toRemove + 1)];
							}}>Remove</Button>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}

		<div class="add-action">
			<Select
				labelText="Action to add"
				bind:value={currentActionKind}
				options={{
					"set_automerge": "Set automerge",
					"set_needed_reviewers": "Set needed reviewers",
					"set_qa_enabled": "Set QA enabled",
					"set_checks_enabled": "Set checks enabled",
				}}
			/>

			{#if currentActionKind == "set_automerge"}
				<CheckBox labelText="Enabled" bind:checked={currentActionValue} />
			{:else if currentActionKind == "set_needed_reviewers"}
				<NumberInput labelText="Count" bind:value={currentActionValue} />
			{:else if currentActionKind == "set_qa_enabled" || currentActionKind == "set_checks_enabled"}
				<CheckBox labelText="Enabled" bind:checked={currentActionValue} />
			{/if}

			<Button disabled={currentActionValue === ""} on:click={() => {
				actions = [...actions, {[currentActionKind]: currentActionValue}];
				currentActionValue = "";
			}}>Add</Button>
		</div>
	</FieldSet>

	<Button type="submit" disabled={conditions.length == 0 || actions.length == 0 || name.trim() == ""}>Submit</Button>
</Form>

<style lang="scss">
	.add-condition {
		display: flex;
		flex-direction: row;
		gap: 0.25rem;

		align-items: flex-end;
		justify-content: stretch;
	}

	.add-action {
		display: flex;
		flex-direction: row;
		gap: 0.25rem;

		align-items: flex-end;
	}
</style>
