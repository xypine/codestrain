import { get_strains } from '$lib/backend';
import type { PageLoad } from './$types';

export const load = (async ({parent, fetch}) => {
    const data = await parent();
    if(!("user" in data) || !data.user) return {};
    const user_strains = await get_strains(data.user.id, fetch);
    return {
        user: {
            ...data.user,
            strains: user_strains
        }
    }
}) satisfies PageLoad;