const API_BASE_URL =
	import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080'

export type AuthUser = {
	id?: number
	user_id?: number
	email?: string
	role?: string
	fullName?: string
	avatarUrl?: string | null
	auth_provider?: string
	authProvider?: string
}

function saveUserData(data: any) {
	if (!data?.token) return

	const normalizedUser: AuthUser = {
		id: data.user_id ?? data.id,
		user_id: data.user_id ?? data.id,
		email: data.email ?? '',
		role: data.role ?? 'user',
		fullName: data.fullName ?? data.full_name ?? '',
		avatarUrl: data.avatarUrl ?? data.avatar_url ?? null,
		auth_provider: data.auth_provider,
		authProvider: data.authProvider,
	}

	localStorage.setItem('token', data.token)
	localStorage.setItem(
		'user_id',
		String(normalizedUser.id ?? normalizedUser.user_id ?? ''),
	)
	localStorage.setItem('user_role', normalizedUser.role || 'user')
	localStorage.setItem('user', JSON.stringify(normalizedUser))

	window.dispatchEvent(new Event('user-updated'))
}

export const authApi = {
	register: async (email: string, password: string) => {
		const response = await fetch(`${API_BASE_URL}/api/auth/register`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email, password }),
		})

		if (!response.ok) {
			const err = await response.text().catch(() => '')
			throw new Error(err || `Registration failed: ${response.status}`)
		}

		const data = await response.json()
		saveUserData(data)
		return data
	},

	login: async (email: string, password: string) => {
		const response = await fetch(`${API_BASE_URL}/api/auth/login`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email, password }),
		})

		if (!response.ok) {
			throw new Error(`Login failed: ${response.status}`)
		}

		const data = await response.json()
		saveUserData(data)
		return data
	},

	logout: () => {
		localStorage.removeItem('token')
		localStorage.removeItem('user_id')
		localStorage.removeItem('user_role')
		localStorage.removeItem('user')
		window.dispatchEvent(new Event('user-updated'))
		window.location.href = '/'
	},

	getUser: (): AuthUser | null => {
		try {
			const raw = localStorage.getItem('user')
			return raw ? JSON.parse(raw) : null
		} catch {
			return null
		}
	},

	getUserRole: () => {
		const user = authApi.getUser()
		return user?.role || localStorage.getItem('user_role')
	},

	getToken: () => localStorage.getItem('token'),

	getUserId: () => {
		const user = authApi.getUser()
		return (
			user?.id?.toString() ||
			user?.user_id?.toString() ||
			localStorage.getItem('user_id')
		)
	},

	isAuthenticated: () => !!localStorage.getItem('token'),

	syncProfile: async () => {
		const token = authApi.getToken()
		if (!token) return null

		const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
			headers: {
				Authorization: `Bearer ${token}`,
			},
		})

		if (!response.ok) {
			if (response.status === 401) {
				authApi.logout()
			}
			throw new Error(`Profile sync failed: ${response.status}`)
		}

		const data = await response.json()

		const normalizedUser: AuthUser = {
			id: data.id,
			user_id: data.id,
			email: data.email,
			role: data.role,
			fullName: data.fullName ?? data.full_name ?? '',
			avatarUrl: data.avatarUrl ?? data.avatar_url ?? null,
			auth_provider: data.auth_provider,
			authProvider: data.auth_provider,
		}

		localStorage.setItem('user', JSON.stringify(normalizedUser))
		localStorage.setItem('user_id', String(data.id))
		localStorage.setItem('user_role', data.role || 'user')
		window.dispatchEvent(new Event('user-updated'))

		return normalizedUser
	},
}
