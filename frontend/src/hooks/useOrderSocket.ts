import { useEffect, useState } from 'react'
import type { WsOrderStatusMessage } from '../types'

const WS_BASE_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:8083'

export const useOrderSocket = (orderId: number | null) => {
	const [wsMessage, setWsMessage] = useState<WsOrderStatusMessage | null>(null)
	const [isConnected, setIsConnected] = useState(false)

	useEffect(() => {
		if (orderId === null) {
			setWsMessage(null)
			setIsConnected(false)
			return
		}

		setWsMessage(null)
		setIsConnected(false)

		const baseUrl = WS_BASE_URL.replace(/\/ws\/?$/, '').replace(/\/$/, '')
		const ws = new WebSocket(`${baseUrl}/ws?order_id=${orderId}`)

		ws.onopen = () => setIsConnected(true)

		ws.onmessage = event => {
			try {
				setWsMessage(JSON.parse(event.data) as WsOrderStatusMessage)
			} catch {}
		}

		ws.onerror = () => setIsConnected(false)
		ws.onclose = () => setIsConnected(false)

		return () => {
			ws.onopen = null
			ws.onmessage = null
			ws.onerror = null
			ws.onclose = null

			if (
				ws.readyState === WebSocket.OPEN ||
				ws.readyState === WebSocket.CONNECTING
			) {
				ws.close()
			}
		}
	}, [orderId])

	return { wsMessage, isConnected }
}
