import { authApi } from './auth'

const API_BASE_URL =
	import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080'

export interface OrderDto {
	id: number
	user_id?: number
	amount: number
	status: string
	created_at?: string
}

const getAuthToken = () => {
	const token = authApi.getToken()

	if (!token) {
		window.location.href = '/login'
		throw new Error('Необхідна авторизація')
	}

	return token
}

const getHeaders = (token: string): HeadersInit => ({
	'Content-Type': 'application/json',
	Authorization: `Bearer ${token}`,
})

const parseJson = <T>(text: string, fallback: T): T => {
	return text ? (JSON.parse(text) as T) : fallback
}

export const ordersApi = {
	createOrder: async (data: { amount: number }) => {
		const token = getAuthToken()

		const response = await fetch(`${API_BASE_URL}/api/orders`, {
			method: 'POST',
			headers: getHeaders(token),
			body: JSON.stringify(data),
		})

		const text = await response.text()

		if (!response.ok) {
			throw new Error(
				`Помилка створення замовлення: ${response.status} ${text}`,
			)
		}

		return parseJson(text, null)
	},

	getOrders: async (): Promise<OrderDto[]> => {
		const token = getAuthToken()

		const response = await fetch(`${API_BASE_URL}/api/orders`, {
			method: 'GET',
			headers: getHeaders(token),
		})

		const text = await response.text()

		if (!response.ok) {
			throw new Error(
				`Помилка завантаження замовлень: ${response.status} ${text}`,
			)
		}

		return parseJson<OrderDto[]>(text, [])
	},

	getOrderById: async (orderId: number): Promise<OrderDto> => {
		const token = getAuthToken()

		const response = await fetch(`${API_BASE_URL}/api/orders/${orderId}`, {
			method: 'GET',
			headers: getHeaders(token),
		})

		const text = await response.text()

		if (!response.ok) {
			throw new Error(
				`Помилка завантаження замовлення: ${response.status} ${text}`,
			)
		}

		return parseJson<OrderDto>(text, {} as OrderDto)
	},
}
