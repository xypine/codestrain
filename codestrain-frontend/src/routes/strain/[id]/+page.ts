import { get_strain } from '$lib/backend';
import type { PageLoad } from './$types';

export const load = (async ({fetch, params}) => {
    const strain = await get_strain(params.id, fetch);
    return {
        strain,
    };
}) satisfies PageLoad;