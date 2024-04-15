import { authToken } from "../store";
import type { Account, ExtendedExternalAccount, ExtendedRepository, PullRequestRule } from "./types";

const URL = "http://localhost:8008";
let token: string | null = null;

authToken.subscribe((value) => token = value);

function buildRoute(path: string): string {
	return `${URL}${path}`;
}

export class ApiClient {
	accounts: ApiAccountClient
	repositories: ApiRepositoryClient
	externalAccounts: ApiExternalAccountClient

	constructor() {
		this.accounts = new ApiAccountClient();
		this.repositories = new ApiRepositoryClient();
		this.externalAccounts = new ApiExternalAccountClient();
	}
}

export class ApiRepositoryClient {
	async list(): Promise<ExtendedRepository[]> {
		const response = await fetch(buildRoute("/admin/repositories/"), {
			method: "GET",
			headers: {
				"Authorization": `Bearer ${token}`
			}
		})

		return await response.json()
	}

	withId(repositoryId: number): ApiRepositoryDetailClient {
		return new ApiRepositoryDetailClient(repositoryId);
	}
}

export class ApiRepositoryDetailClient {
	pullRequestRules: ApiRepositoryPullRequestRuleClient

	constructor(repositoryId: number) {
		this.pullRequestRules = new ApiRepositoryPullRequestRuleClient(repositoryId);
	}
}

export class ApiRepositoryPullRequestRuleClient {
	repositoryId: number;

	constructor(repositoryId: number) {
		this.repositoryId = repositoryId;
	}

	async create(rule: PullRequestRule): Promise<PullRequestRule> {
		const response = await fetch(buildRoute(`/admin/repositories/${this.repositoryId}/pull-request-rules/`), {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				"Authorization": `Bearer ${token}`
			},
			body: JSON.stringify(rule)
		})

		return await response.json()
	}

	async delete(rule: PullRequestRule): Promise<Response> {
		return await fetch(buildRoute(`/admin/repositories/${this.repositoryId}/pull-request-rules/${encodeURI(rule.name)}/`), {
			method: "DELETE",
			headers: {
				"Authorization": `Bearer ${token}`
			}
		})
	}
}

export class ApiAccountClient {
	async list(): Promise<Account[]> {
		const response = await fetch(buildRoute("/admin/accounts/"), {
			method: "GET",
			headers: {
				"Authorization": `Bearer ${token}`
			}
		})

		return await response.json()
	}
}

export class ApiExternalAccountClient {
	async list(): Promise<ExtendedExternalAccount[]> {
		const response = await fetch(buildRoute("/admin/external-accounts/"), {
			method: "GET",
			headers: {
				"Authorization": `Bearer ${token}`
			}
		})

		return await response.json()
	}
}
