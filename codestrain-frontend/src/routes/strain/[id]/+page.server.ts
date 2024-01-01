import { create_strain_version, delete_strain, run_strain_version } from '$lib/backend';
import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	create_version: async ({request, cookies, fetch, params}) => {
		// TODO create strain version
        const data = await request.formData();
        const code = data.get('code');
        const wasm = data.get('wasm');

        if(!code || !wasm) {
            return fail(400, {
                message: 'code and wasm are required',
                error: true
            });
        }

        let result;
        try {
            result = await create_strain_version(params.id, code + "", wasm + "", cookies.get('session')!, fetch);
        }
        catch(e) {
            console.error(e);
            return fail(500, {
                message: 'Failed to create strain version',
                error: true
            });
        }
        throw redirect(302, `/strain/${params.id}?version=${result.id}`);
	},
    run_version: async ({request, cookies, fetch, params}) => {
        // TODO run strain version
        const data = await request.formData();
        const strain_id = params.id;
        const version_id = data.get('version_id');

        if(!version_id) {
            return fail(400, {
                message: 'version_id is required',
                error: true
            });
        }

        let result;
        try {
            result = await run_strain_version(strain_id, version_id + "", cookies.get('session')!, fetch);
        }
        catch(e) {
            console.error(e);
            return fail(500, {
                message: 'Failed run strain version',
                error: true
            });
        }
        return {
            result
        }
    },
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