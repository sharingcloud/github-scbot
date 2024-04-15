import type { PageLoad } from './$types';

export const load: PageLoad = ({ params }) => {
	return {
		repositoryId: parseInt(params.repository_id)
	};
};
