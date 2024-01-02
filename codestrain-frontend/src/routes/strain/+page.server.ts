import { create_strain } from '$lib/backend';
import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	default: async ({request, cookies, fetch}) => {
		// TODO create strain
        const data = await request.formData();
        const name = data.get('name');
        const description = data.get('description');
        const code = data.get('code');
        const wasm = data.get('wasm');

        if(!name) {
            return fail(400, {
                message: 'name is required',
                error: true
            });
        }


        if(!code || !wasm) {
            return fail(400, {
                message: 'code and wasm are required',
                error: true
            });
        }

        let result;
        try {
             result = await create_strain(name + "", description ? description + "" : null, code + "", wasm + "", cookies.get('session')!, fetch);
        }
        catch(e) {
            console.error(e);
            return fail(500, {
                message: 'Failed to create strain',
                error: true
            });
        }
        return redirect(302, `/strain/${result.id}`);
	}
} satisfies Actions;