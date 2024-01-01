import { BACKEND_URL } from '$lib/backend';
import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	default: async ({request}) => {
		// TODO register user
        const data = await request.formData();
        const name = data.get('name');
        const email = data.get('email');
        const password = data.get('password');

        if(!name || !email || !password) {
            return fail(400, {
                message: 'Name, email and password are required',
                error: true
            });
        }

        const response = await fetch(`${BACKEND_URL}/user`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({name, email, password})
        });
        if(!response.ok) {
            const error = await response.text();
            console.error(error);
            return fail(401, {
                message: 'Failed to register',
                error: true
            });
        }
        return redirect(302, '/login');
	},
} satisfies Actions;