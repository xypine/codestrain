import { get_battle, get_strain } from '$lib/backend';
import type { PageLoad } from './$types';

export const load = (async ({params, fetch}) => {
    const battle = await get_battle(params.id, fetch);
    const strain_a = await get_strain(battle.strain_a, fetch);
    const strain_b = await get_strain(battle.strain_b, fetch);
    return {
        battle,
        strain_a,
        strain_b
    };
}) satisfies PageLoad;