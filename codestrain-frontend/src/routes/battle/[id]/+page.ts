import { get_battle } from '$lib/backend';
import type { PageLoad } from './$types';

export const load = (async ({params, fetch}) => {
    const battle = await get_battle(params.id, fetch);
    return {
        battle
    };
}) satisfies PageLoad;