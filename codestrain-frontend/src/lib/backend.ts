export const BACKEND_URL = 'http://localhost:8000' as const;

export type User = {
    id: string;
    name: string;
    email: string;
    admin: boolean;
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
    description: string | null;
    created_at: string;
    updated_at: string;
}

export async function get_strains(user_id?: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/strain?creator_id=${user_id}`);
    if(!response.ok) throw new Error(`Failed to fetch strains (${response.status})`);
    return response.json() as Promise<Strain[]>;
}

export async function create_strain(name: string, description: string | null, code: string, wasm: string, token: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const body = {
        name,
        description,
        code,
        wasm
    };
    const response = await fetch(`${BACKEND_URL}/strain`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify(body)
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

export type BattleMeta = {
    id: string;
    arena_size: number;
    strain_a: string;
    strain_b: string;
    winner: string | null;
    score_a: number;
    score_b: number;
}
export type BattleResult = BattleMeta & {
    log: {
        player: boolean;
        x: number;
        y: number;
        allowed: boolean;
    }[];
}

export async function request_battle(strain_a: string, strain_b: string, token: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/battle`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({strain_a, strain_b})
    });
    if(!response.ok) throw new Error(`Failed to request battle (${response.status})`);
    return response.json() as Promise<BattleResult>;
}

export async function get_battle(id: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/battle/${id}`);
    if(!response.ok) throw new Error(`Failed to fetch battle (${response.status})`);
    return response.json() as Promise<BattleResult>;
}

export async function get_battles(custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/battle`);
    if(!response.ok) throw new Error(`Failed to fetch battles (${response.status})`);
    return response.json() as Promise<BattleMeta[]>;
}

export type StrainWithExtra = Strain & {
    code: string;
    wasm_size: number;
    wasm_hash: string;
}


export async function get_strain(id: string, custom_fetch?: typeof fetch) {
    const fetch = custom_fetch || window.fetch;
    const response = await fetch(`${BACKEND_URL}/strain/${id}`);
    if(!response.ok) throw new Error(`Failed to fetch strain (${response.status})`);
    return response.json() as Promise<StrainWithExtra>;
}