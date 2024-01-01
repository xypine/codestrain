import { BACKEND_URL, type Session } from '$lib/backend';
import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	default: async ({request, cookies}) => {
		// TODO login user
        const data = await request.formData();
        const email = data.get('email');
        const password = data.get('password');

        if(!email || !password) {
            return fail(400, {
                message: 'Email and password are required',
                error: true
            });
        }

        const response = await fetch(`${BACKEND_URL}/login`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({email, password})
        });
        if(!response.ok) {
            const error = await response.text();
            console.error(error);
            return fail(401, {
                message: 'Failed to login',
                error: true
            });
        }
        const session: Session = await response.json();
        try {
            cookies.set('session', session.token, {
                path: '/',
                maxAge: 60 * 60 * 24 * 30, // 30 days,
                secure: true,
                httpOnly: true
            });
        }
        catch {
            return fail(500, {
                message: 'Failed to save session',
                error: true
            });
        }
        return redirect(302, "/");
	},
} satisfies Actions;