import { BACKEND_URL, type User } from '$lib/backend';
import type { LayoutServerLoad } from './$types';

export const load = (async ({cookies}) => {
    const session = cookies.get('session');
    if(!session) return {};
    const validation_response = await fetch(`${BACKEND_URL}/me`, {
        headers: {
            'Authorization': `Bearer ${session}`,
        }
    });
    if(!validation_response.ok) {
        cookies.delete('session', {
            path: '/'
        });
        return {};
    };
    return {
        session,
        user: await validation_response.json()
    } as {
        session: string,
        user: User
    };
}) satisfies LayoutServerLoad;