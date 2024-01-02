import { request_battle } from "$lib/backend";
import { fail, redirect } from "@sveltejs/kit";
import type { Actions } from "./$types";

export const actions = {
    default: async ({ request, cookies }) => {
        const token = cookies.get('session');
        const data = await request.formData();
        const strain_a = data.get('strain_a');
        const strain_b = data.get('strain_b');
        if (!strain_a || !strain_b) {
            return {
                status: 400,
                body: {
                    message: 'strain_a and strain_b are required',
                    error: true
                }
            };
        }

        let result;
        try {
             result = await request_battle(strain_a + "", strain_b + "", token!, fetch);
        }
        catch(e) {
            console.error(e);
            return fail(500, {
                message: 'Failed to run battle',
                error: true
            });
        }
        return redirect(302, `/battle/${result.id}`);
    }
} satisfies Actions;