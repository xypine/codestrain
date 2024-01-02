import { get_battles, get_strains } from "$lib/backend";
import type { PageLoad } from "./$types";

export const load = (async ({ fetch }) => {
    const strains = await get_strains(undefined, fetch);
    const battles = await get_battles(fetch);
    return {
        strains,
        battles
    }
}) satisfies PageLoad;