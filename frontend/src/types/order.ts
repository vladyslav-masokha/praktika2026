export interface CreateOrderRequest {
	amount: number
}

export interface OrderResponse {
	id: number
	user_id: number
	amount: number
	status: 'PENDING' | 'CONFIRMED' | 'FAILED'
	created_at: string
	updated_at?: string
}
