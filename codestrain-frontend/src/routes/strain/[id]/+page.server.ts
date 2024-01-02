import { delete_strain } from '$lib/backend';
import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
    delete: async ({cookies, fetch, params}) => {
        try {
            await delete_strain(params.id, cookies.get('session')!, fetch);
        }
        catch(e) {
            console.error(e);
            return fail(500, {
                message: 'Failed to delete strain',
                error: true
            });
        }
        throw redirect(302, `/profile`);
    }
} satisfies Actions;