import { CheckCircle2, CreditCard, Loader2, XCircle } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { ordersApi } from '../api/orders'
import { useOrderSocket } from '../hooks/useOrderSocket'

interface Props {
	orderId: number | null
	onClose: () => void
}

type FinalStatus = 'CONFIRMED' | 'FAILED' | null

const normalizeFinalStatus = (status?: string | null): FinalStatus => {
	const normalized = String(status ?? '').toUpperCase()

	if (normalized === 'CONFIRMED' || normalized === 'PAID') {
		return 'CONFIRMED'
	}

	if (normalized === 'FAILED') {
		return 'FAILED'
	}

	return null
}

export const LiveCheckoutModal = ({ orderId, onClose }: Props) => {
	const { wsMessage, isConnected } = useOrderSocket(orderId)
	const [polledStatus, setPolledStatus] = useState<FinalStatus>(null)
	const [polledMessage, setPolledMessage] = useState('')
	const isFinalized = useRef(false)

	useEffect(() => {
		if (!orderId) {
			setPolledStatus(null)
			setPolledMessage('')
			isFinalized.current = false
			return
		}

		setPolledStatus(null)
		setPolledMessage('')
		isFinalized.current = false

		const checkOrderStatus = async () => {
			if (isFinalized.current) {
				return
			}

			try {
				const order = await ordersApi.getOrderById(orderId)
				const status = normalizeFinalStatus(order.status)

				if (status === 'CONFIRMED') {
					setPolledStatus('CONFIRMED')
					setPolledMessage('Оплату успішно підтверджено.')
					isFinalized.current = true
				}

				if (status === 'FAILED') {
					setPolledStatus('FAILED')
					setPolledMessage('Оплату не вдалося виконати.')
					isFinalized.current = true
				}
			} catch {}
		}

		checkOrderStatus()
		const intervalId = window.setInterval(checkOrderStatus, 3000)

		return () => {
			isFinalized.current = true
			window.clearInterval(intervalId)
		}
	}, [orderId])

	const wsStatus = normalizeFinalStatus(wsMessage?.status)
	const finalStatus = wsStatus ?? polledStatus
	const isSuccess = finalStatus === 'CONFIRMED'
	const isFailed = finalStatus === 'FAILED'
	const isWaiting = !isSuccess && !isFailed

	useEffect(() => {
		if (isSuccess || isFailed) {
			isFinalized.current = true
		}
	}, [isSuccess, isFailed])

	if (!orderId) {
		return null
	}

	const finalMessage =
		wsMessage?.message ||
		polledMessage ||
		(isSuccess
			? 'Ваше замовлення прийнято до обробки.'
			: 'Сталася помилка під час обробки замовлення.')

	return (
		<div className='fixed inset-0 z-[100] flex items-center justify-center bg-[#030712]/80 p-4 backdrop-blur-md transition-all duration-500'>
			<div className='relative w-full max-w-md overflow-hidden rounded-[2.5rem] border border-slate-800/60 bg-slate-900/90 p-8 text-center shadow-2xl sm:p-10'>
				<div className='absolute -right-24 -top-24 h-48 w-48 rounded-full bg-blue-600/10 blur-[80px]' />

				<header className='relative z-10 mb-8'>
					<h2 className='text-2xl font-black uppercase tracking-tight text-white'>
						Замовлення{' '}
						<span className='font-mono text-blue-500'>#{orderId}</span>
					</h2>
				</header>

				<div className='relative z-10 flex min-h-[240px] flex-col items-center justify-center'>
					{isWaiting && (
						<div className='animate-in fade-in zoom-in flex flex-col items-center duration-300'>
							<div className='relative mb-8'>
								<Loader2
									className='h-20 w-20 animate-spin text-blue-600'
									strokeWidth={1.5}
								/>
								<CreditCard className='absolute left-1/2 top-1/2 h-8 w-8 -translate-x-1/2 -translate-y-1/2 text-slate-400' />
							</div>
							<p className='mb-2 text-lg font-bold text-white'>
								{isConnected ? 'Обробка оплати...' : 'Підключення...'}
							</p>
							<p className='max-w-[240px] text-sm leading-relaxed text-slate-500'>
								Будь ласка, зачекайте.
							</p>
						</div>
					)}

					{isSuccess && (
						<div className='animate-in slide-in-from-bottom-4 flex flex-col items-center duration-500'>
							<div className='mb-8 flex h-24 w-24 items-center justify-center rounded-3xl border border-emerald-500/20 bg-emerald-500/10 text-emerald-400 shadow-lg shadow-emerald-500/5'>
								<CheckCircle2 size={48} strokeWidth={2.5} />
							</div>
							<h3 className='mb-3 text-3xl font-black uppercase tracking-tighter text-white'>
								Оплачено
							</h3>
							<p className='px-4 text-sm font-medium leading-relaxed text-slate-400'>
								{finalMessage}
							</p>
						</div>
					)}

					{isFailed && (
						<div className='animate-in slide-in-from-bottom-4 flex flex-col items-center duration-500'>
							<div className='mb-8 flex h-24 w-24 items-center justify-center rounded-3xl border border-red-500/20 bg-red-500/10 text-red-500 shadow-lg shadow-red-500/5'>
								<XCircle size={48} strokeWidth={2.5} />
							</div>
							<h3 className='mb-3 text-3xl font-black uppercase tracking-tighter text-white'>
								Помилка
							</h3>
							<p className='px-4 text-sm font-medium leading-relaxed text-slate-400'>
								{finalMessage}
							</p>
						</div>
					)}
				</div>

				<footer className='relative z-10 mt-10'>
					{!isWaiting && (
						<button
							onClick={onClose}
							className='w-full rounded-2xl bg-blue-600 py-5 text-xs font-black uppercase tracking-widest text-white shadow-lg shadow-blue-600/20 transition-all hover:bg-blue-500 active:scale-[0.98]'
						>
							Повернутися до покупок
						</button>
					)}
				</footer>
			</div>
		</div>
	)
}
