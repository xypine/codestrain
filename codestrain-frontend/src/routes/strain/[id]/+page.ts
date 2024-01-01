import { get_strain_with_versions, get_strain_version } from '$lib/backend';
import type { PageLoad } from './$types';

export const load = (async ({fetch, params, url}) => {
    const strain_details = await get_strain_with_versions(params.id, fetch);
    let selected_version = undefined;
    const url_version = url.searchParams.get('version');
    if (url_version != null) {
        selected_version = await get_strain_version(params.id, url_version, fetch);
    }
    return {
        strain: strain_details,
        selected_version
    };
}) satisfies PageLoad;