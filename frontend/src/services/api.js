export function getApiKey() {
    return localStorage.getItem('api_key');
}

async function apiRequest(url, options = {}) {
    const apiKey = getApiKey();

    if (!apiKey) {
        throw new Error('Not authenticated. Please login first.');
    }

    const headers = {
        'Content-Type': 'application/json',
        'X-API-Key': apiKey,
        ...options.headers,
    };

    const response = await fetch(url, {
        ...options,
        headers,
    });

    if (response.status === 401) {
        localStorage.removeItem('api_key');
        localStorage.removeItem('username');
        localStorage.removeItem('user_id');
        localStorage.removeItem('role');

        window.location.href = '/';
        throw new Error('Session expired. Please login again.');
    }

    return response;
}
export async function get(url, params = {}) {
    const queryString = new URLSearchParams(params).toString();
    const fullUrl = queryString ? `${url}?${queryString}` : url;

    const response = await apiRequest(fullUrl, {
        method: 'GET',
    });

    if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        throw new Error(error.error || error.message || 'Request failed');
    }

    return response.json();
}

export async function post(url, data) {
    const response = await apiRequest(url, {
        method: 'POST',
        body: JSON.stringify(data),
    });

    if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        throw new Error(error.error || error.message || 'Request failed');
    }

    return response.json();
}

export async function del(url) {
    const response = await apiRequest(url, {
        method: 'DELETE',
    });

    if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        throw new Error(error.error || error.message || 'Request failed');
    }

    const text = await response.text();
    return text ? JSON.parse(text) : {};
}