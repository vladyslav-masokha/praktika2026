import {
	AlertCircle,
	CheckCircle2,
	Clock,
	ExternalLink,
	ShoppingBag,
} from 'lucide-react'
import React, { useEffect, useState } from 'react'
import { ordersApi, type OrderDto } from '../api/orders'

export const OrdersPage: React.FC = () => {
	const [orders, setOrders] = useState<OrderDto[]>([])
	const [isLoading, setIsLoading] = useState(true)
	const [error, setError] = useState<string | null>(null)

	useEffect(() => {
		const fetchOrders = async () => {
			try {
				const data = await ordersApi.getOrders()
				setOrders(data)
			} catch (err) {
				const message = err instanceof Error ? err.message : 'Невідома помилка'
				setError(message)
			} finally {
				setIsLoading(false)
			}
		}

		fetchOrders()
	}, [])

	const getStatusStyle = (status: string) => {
		switch (status.toLowerCase()) {
			case 'paid':
			case 'confirmed':
				return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
			case 'pending':
				return 'bg-amber-500/10 text-amber-400 border-amber-500/20'
			case 'cancelled':
				return 'bg-red-500/10 text-red-400 border-red-500/20'
			default:
				return 'bg-blue-500/10 text-blue-400 border-blue-500/20'
		}
	}

	return (
		<div className='min-h-screen bg-[#030712] text-slate-200 py-12 px-6'>
			<div className='max-w-4xl mx-auto'>
				<div className='flex items-center justify-between mb-12'>
					<div className='flex items-center gap-4'>
						<div className='p-3 bg-blue-600/20 rounded-2xl border border-blue-500/30'>
							<ShoppingBag className='text-blue-500 w-8 h-8' />
						</div>
						<div>
							<h2 className='text-4xl font-black text-white tracking-tight uppercase'>
								Історія
							</h2>
							<p className='text-slate-500 text-sm font-medium'>
								Ваші замовлення та транзакції
							</p>
						</div>
					</div>
				</div>

				{isLoading ? (
					<div className='space-y-4'>
						{[1, 2, 3].map(i => (
							<div
								key={i}
								className='h-28 bg-slate-900/40 border border-slate-800/50 animate-pulse rounded-3xl'
							/>
						))}
					</div>
				) : error ? (
					<div className='flex items-center gap-3 p-5 bg-red-500/5 border border-red-500/20 rounded-2xl text-red-400 backdrop-blur-md'>
						<AlertCircle size={22} />
						<p className='font-medium'>{error}</p>
					</div>
				) : orders.length === 0 ? (
					<div className='text-center py-24 bg-slate-900/20 border-2 border-dashed border-slate-800/50 rounded-[2.5rem]'>
						<p className='text-slate-500 font-medium text-lg'>
							У вас ще немає замовлень. <br />
							<span className='text-blue-500 cursor-pointer hover:underline'>
								Час щось придбати!
							</span>
						</p>
					</div>
				) : (
					<div className='grid gap-5'>
						{orders.map(order => (
							<div
								key={order.id}
								className='group bg-slate-900/40 border border-slate-800/60 p-6 rounded-[2rem] flex flex-col sm:flex-row justify-between items-start sm:items-center hover:bg-slate-900/60 hover:border-blue-500/30 transition-all duration-300 cursor-default backdrop-blur-sm'
							>
								<div className='flex items-center gap-5 mb-5 sm:mb-0'>
									<div className='p-4 bg-slate-800/50 rounded-2xl group-hover:bg-blue-600/10 transition-colors border border-slate-700/50 group-hover:border-blue-500/20'>
										<Clock
											className='text-slate-400 group-hover:text-blue-400 transition-colors'
											size={26}
										/>
									</div>
									<div>
										<div className='flex items-center gap-2 mb-1'>
											<h3 className='text-white font-bold text-xl'>
												Замовлення #{order.id}
											</h3>
											<ExternalLink
												size={14}
												className='text-slate-600 opacity-0 group-hover:opacity-100 transition-opacity'
											/>
										</div>
										<p className='text-slate-500 text-sm font-medium'>
											{order.created_at
												? new Date(order.created_at).toLocaleString('uk-UA', {
														day: 'numeric',
														month: 'long',
														year: 'numeric',
														hour: '2-digit',
														minute: '2-digit',
													})
												: 'Дата недоступна'}
										</p>
									</div>
								</div>

								<div className='flex items-center justify-between w-full sm:w-auto gap-8'>
									<div className='text-right'>
										<p className='text-2xl font-mono font-black text-white'>
											{order.amount.toLocaleString()} ₴
										</p>
										<p className='text-[10px] uppercase tracking-[0.2em] text-slate-600 font-bold'>
											Сума
										</p>
									</div>
									<div
										className={`flex items-center gap-2 px-5 py-2 rounded-xl border text-[11px] font-black uppercase tracking-tighter ${getStatusStyle(order.status)} transition-all duration-300 shadow-lg shadow-black/20`}
									>
										<CheckCircle2 size={14} />
										{order.status}
									</div>
								</div>
							</div>
						))}
					</div>
				)}
			</div>
		</div>
	)
}
