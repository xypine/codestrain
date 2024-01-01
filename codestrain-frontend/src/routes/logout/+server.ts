import { BACKEND_URL } from "$lib/backend";
import { redirect, type RequestHandler } from "@sveltejs/kit";

export const GET = (async ({cookies}) => {
    const session = cookies.get('session');
    if(!session) return redirect(302, '/login');
    /* The cookie should be deleted automatically
    cookies.delete('session', {
        path: '/'
    });*/
    await fetch(`${BACKEND_URL}/logout`, {
        method: 'POST',
        body: JSON.stringify({token: session}),
    });
    return redirect(302, '/login');
}) satisfies RequestHandler;