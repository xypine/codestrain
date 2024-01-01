export const BACKEND_URL = 'http://localhost:8000' as const;

export type User = {
    id: string;
    name: string;
    email: string;
    password?: string;
    created_at: string;
    updated_at: string;
}

export type Session = {
    id: string;
    creator_id: string;
    token: string;
    created_at: string;
    updated_at: string;
}

export type Strain = {
    id: string;
    creator_id: string;
    name: string;
    created_at: string;
    updated_at: string;
}

export async function get_strains_by_user(user_id: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/strain?creator_id=${user_id}`);
    if(!response.ok) throw new Error(`Failed to fetch strains (${response.status})`);
    return response.json() as Promise<Strain[]>;
}

export async function create_strain(name: string, token: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/strain`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({name})
    });
    if(!response.ok) throw new Error(`Failed to create strain (${response.status})`);
    return response.json() as Promise<Strain>;
}

export async function delete_strain(id: string, token: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/strain/${id}`, {
        method: 'DELETE',
        headers: {
            'Authorization': `Bearer ${token}`
        }
    });
    if(!response.ok) throw new Error(`Failed to delete strain (${response.status})`);
}

export async function create_strain_version(strain_id: string, code: string, wasm: string, token: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/strain/${strain_id}/version`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({code, wasm})
    });
    if(!response.ok) throw new Error(`Failed to create strain version (${response.status})`);
    return response.json() as Promise<StrainVersionMeta>;
}

export async function run_strain_version(strain_id: string, version_id: string, token: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/strain/${strain_id}/version/${version_id}/run`, {
        headers: {
            'Authorization': `Bearer ${token}`
        },
    });
    if(!response.ok) throw new Error(`Failed to create strain version (${response.status})`);
    return response.text() as Promise<string>;
}

export type StrainVersionMeta = {
    id: string;
    strain_id: string;
    created_at: string;
    updated_at: string;
}

export type StrainWithVersions = Strain & {
    versions: StrainVersionMeta[];
}


export async function get_strain_with_versions(id: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/strain/${id}`);
    if(!response.ok) throw new Error(`Failed to fetch strain (${response.status})`);
    return response.json() as Promise<StrainWithVersions>;
}

export type StrainVersion = StrainVersionMeta & {
    code: string;
}
export async function get_strain_version(strain_id: string, id: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/strain/${strain_id}/version/${id}`);
    if(!response.ok) throw new Error(`Failed to fetch strain version (${response.status})`);
    return response.json() as Promise<StrainVersion>;
}