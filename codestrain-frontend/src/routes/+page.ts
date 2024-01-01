import { BACKEND_URL, type User } from '$lib/backend';
import type { PageLoad } from './$types';

export const load = (async ({fetch}) => {
    const users_response = await fetch(`${BACKEND_URL}/user`);
    if(!users_response.ok) throw new Error('Failed to fetch users');
    const users: User[] = await users_response.json();
    return {
        users
    };
}) satisfies PageLoad;