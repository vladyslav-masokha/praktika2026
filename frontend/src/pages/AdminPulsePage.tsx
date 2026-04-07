import {
	Activity,
	AlertTriangle,
	Clock3,
	Package,
	Plus,
	RefreshCw,
	ShieldCheck,
	Tag,
	Trash2,
	Users,
	Zap,
} from 'lucide-react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { authApi } from '../api/auth'
import { useOrderSocket } from '../hooks/useOrderSocket'

type SystemEvent = {
	order_id: number
	status: string
	event_type: string
	message: string
	timestamp: string
}

type AdminStats = {
	totalUsers: number
	totalProducts: number
	totalOrders: number
	totalVisits: number
	onlineUsers: number
}

type AdminUser = {
	id: number
	email: string
	role: string
	full_name?: string | null
	avatar_url?: string | null
	auth_provider: string
	created_at?: string | null
	last_seen_at?: string | null
}

type Product = {
	id: number
	slug: string
	name: string
	description: string
	price: number
	image_url?: string | null
	category?: string | null
	is_active: boolean
}

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL

if (!API_BASE_URL) {
	throw new Error('VITE_API_BASE_URL is not defined')
}

const emptyForm = {
	id: 0,
	slug: '',
	name: '',
	description: '',
	price: 0,
	image_url: '',
	category: '',
	is_active: true,
}

function formatPrice(value: number) {
	return `${value.toLocaleString('uk-UA')} ₴`
}

export const AdminPulsePage = () => {
	const { wsMessage, isConnected } = useOrderSocket(0)
	const chatEndRef = useRef<HTMLDivElement>(null)

	const [events, setEvents] = useState<SystemEvent[]>([])
	const [stats, setStats] = useState<AdminStats | null>(null)
	const [users, setUsers] = useState<AdminUser[]>([])
	const [products, setProducts] = useState<Product[]>([])
	const [loading, setLoading] = useState(true)
	const [saving, setSaving] = useState(false)
	const [error, setError] = useState('')
	const [form, setForm] = useState(emptyForm)
	const [brokenAvatars, setBrokenAvatars] = useState<Record<number, boolean>>(
		{},
	)

	useEffect(() => {
		if (wsMessage) {
			const newEvent: SystemEvent = {
				...wsMessage,
				timestamp: new Date().toLocaleTimeString('uk-UA'),
			}
			setEvents(prev => [newEvent, ...prev].slice(0, 50))
		}
	}, [wsMessage])

	useEffect(() => {
		chatEndRef.current?.scrollIntoView({ behavior: 'smooth' })
	}, [events])

	const loadData = async () => {
		const token = authApi.getToken()

		if (!token) {
			return
		}

		try {
			setLoading(true)
			setError('')

			const headers = { Authorization: `Bearer ${token}` }

			const [statsRes, usersRes, productsRes] = await Promise.all([
				fetch(`${API_BASE_URL}/api/admin/stats`, { headers }),
				fetch(`${API_BASE_URL}/api/admin/users`, { headers }),
				fetch(`${API_BASE_URL}/api/admin/products`, { headers }),
			])

			if (!statsRes.ok || !usersRes.ok || !productsRes.ok) {
				throw new Error('Не вдалося завантажити дані адмінпанелі')
			}

			const [statsData, usersData, productsData] = await Promise.all([
				statsRes.json(),
				usersRes.json(),
				productsRes.json(),
			])

			setStats(statsData)
			setUsers(usersData)
			setProducts(productsData)
		} catch {
			setError('Помилка завантаження даних адмінпанелі')
		} finally {
			setLoading(false)
		}
	}

	useEffect(() => {
		loadData()
	}, [])

	const activeProducts = useMemo(
		() => products.filter(item => item.is_active).length,
		[products],
	)

	const getStatusColor = (status: string) => {
		switch (status.toUpperCase()) {
			case 'PAID':
			case 'CONFIRMED':
				return 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300'
			case 'PENDING':
				return 'border-amber-500/30 bg-amber-500/10 text-amber-300'
			case 'FAILED':
			case 'CANCELLED':
				return 'border-red-500/30 bg-red-500/10 text-red-300'
			default:
				return 'border-blue-500/30 bg-blue-500/10 text-blue-300'
		}
	}

	const resetForm = () => setForm(emptyForm)

	const startEdit = (product: Product) => {
		setForm({
			id: product.id,
			slug: product.slug,
			name: product.name,
			description: product.description,
			price: product.price,
			image_url: product.image_url || '',
			category: product.category || '',
			is_active: product.is_active,
		})

		window.scrollTo({ top: 0, behavior: 'smooth' })
	}

	const handleSubmit = async () => {
		const token = authApi.getToken()

		if (!token) {
			return
		}

		try {
			setSaving(true)
			setError('')

			const payload = {
				slug: form.slug || undefined,
				name: form.name,
				description: form.description,
				price: Number(form.price),
				image_url: form.image_url || null,
				category: form.category || null,
				is_active: form.is_active,
			}

			const response = await fetch(
				form.id
					? `${API_BASE_URL}/api/admin/products/${form.id}`
					: `${API_BASE_URL}/api/admin/products`,
				{
					method: form.id ? 'PUT' : 'POST',
					headers: {
						'Content-Type': 'application/json',
						Authorization: `Bearer ${token}`,
					},
					body: JSON.stringify(payload),
				},
			)

			if (!response.ok) {
				throw new Error('Не вдалося зберегти товар')
			}

			resetForm()
			await loadData()
		} catch {
			setError('Помилка збереження товару')
		} finally {
			setSaving(false)
		}
	}

	const handleDelete = async (id: number) => {
		const token = authApi.getToken()

		if (!token) {
			return
		}

		try {
			const response = await fetch(`${API_BASE_URL}/api/admin/products/${id}`, {
				method: 'DELETE',
				headers: { Authorization: `Bearer ${token}` },
			})

			if (!response.ok) {
				throw new Error('Не вдалося деактивувати товар')
			}

			await loadData()
		} catch {
			setError('Помилка деактивації товару')
		}
	}

	if (loading) {
		return (
			<div className='flex min-h-screen items-center justify-center bg-[#030712] px-4 py-10 text-slate-100'>
				<div className='text-center'>
					<div className='mx-auto mb-4 h-12 w-12 animate-spin rounded-full border-2 border-red-400 border-t-transparent' />
					<p className='text-white/80'>Завантаження...</p>
				</div>
			</div>
		)
	}

	return (
		<div className='min-h-screen bg-[#030712] px-4 py-6 text-slate-100 sm:px-6 sm:py-10'>
			<div className='mx-auto max-w-7xl'>
				<header className='mb-6 rounded-[2rem] border border-slate-800/50 bg-slate-900/30 p-5 backdrop-blur-sm sm:mb-8 sm:p-8'>
					<div className='flex flex-col gap-5 lg:flex-row lg:items-center lg:justify-between'>
						<div className='flex items-start gap-4 sm:gap-5'>
							<div className='rounded-2xl border border-red-500/20 bg-red-600/10 p-4 shadow-lg shadow-red-500/10'>
								<Activity className='h-8 w-8 text-red-500 sm:h-10 sm:w-10' />
							</div>

							<div className='min-w-0'>
								<h1 className='text-3xl font-black uppercase italic tracking-tighter text-white sm:text-5xl'>
									Адмін<span className='text-red-500'>панель</span>
								</h1>
								<p className='mt-2 max-w-2xl text-sm text-slate-400 sm:text-lg'>
									Статистика сайту, користувачі, товари та події системи
								</p>
							</div>
						</div>

						<div className='flex flex-col gap-3 sm:flex-row sm:flex-wrap'>
							<div
								className={`inline-flex items-center justify-center gap-3 rounded-full border px-5 py-3 text-sm font-black uppercase tracking-widest ${
									isConnected
										? 'border-emerald-500/20 bg-emerald-500/10 text-emerald-400'
										: 'animate-pulse border-red-500/20 bg-red-500/10 text-red-400'
								}`}
							>
								<div
									className={`h-3 w-3 rounded-full ${
										isConnected ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'
									}`}
								/>
								{isConnected ? 'Підключено' : "Немає з'єднання"}
							</div>

							<button
								onClick={loadData}
								className='inline-flex items-center justify-center gap-2 rounded-2xl border border-white/10 bg-white/5 px-5 py-3 text-sm font-semibold text-white transition hover:bg-white/10'
							>
								<RefreshCw size={18} />
								Оновити
							</button>
						</div>
					</div>
				</header>

				{error && (
					<div className='mb-6 rounded-2xl border border-red-500/20 bg-red-500/10 px-4 py-3 text-red-200'>
						{error}
					</div>
				)}

				<div className='mb-8 grid grid-cols-2 gap-4 xl:grid-cols-5'>
					<div className='rounded-3xl border border-white/10 bg-white/5 p-5'>
						<p className='text-xs uppercase tracking-[0.2em] text-white/40'>
							Користувачі
						</p>
						<p className='mt-3 text-3xl font-black text-white'>
							{stats?.totalUsers ?? 0}
						</p>
					</div>

					<div className='rounded-3xl border border-white/10 bg-white/5 p-5'>
						<p className='text-xs uppercase tracking-[0.2em] text-white/40'>
							Покупки
						</p>
						<p className='mt-3 text-3xl font-black text-white'>
							{stats?.totalOrders ?? 0}
						</p>
					</div>

					<div className='rounded-3xl border border-white/10 bg-white/5 p-5'>
						<p className='text-xs uppercase tracking-[0.2em] text-white/40'>
							Візити
						</p>
						<p className='mt-3 text-3xl font-black text-white'>
							{stats?.totalVisits ?? 0}
						</p>
					</div>

					<div className='rounded-3xl border border-white/10 bg-white/5 p-5'>
						<p className='text-xs uppercase tracking-[0.2em] text-white/40'>
							Онлайн
						</p>
						<p className='mt-3 text-3xl font-black text-white'>
							{stats?.onlineUsers ?? 0}
						</p>
					</div>

					<div className='col-span-2 rounded-3xl border border-white/10 bg-white/5 p-5 xl:col-span-1'>
						<p className='text-xs uppercase tracking-[0.2em] text-white/40'>
							Активні товари
						</p>
						<p className='mt-3 text-3xl font-black text-white'>
							{activeProducts}
						</p>
					</div>
				</div>

				<div className='grid gap-8 xl:grid-cols-[1.1fr_0.9fr]'>
					<div className='space-y-8'>
						<section className='rounded-[2rem] border border-slate-800 bg-slate-900/40 p-5 shadow-2xl backdrop-blur-md sm:p-6'>
							<div className='mb-6 flex items-center gap-3'>
								<Package className='text-blue-400' />
								<h2 className='text-2xl font-black text-white'>Товари</h2>
							</div>

							<div className='grid gap-3 md:grid-cols-2'>
								<input
									value={form.name}
									onChange={e =>
										setForm(prev => ({ ...prev, name: e.target.value }))
									}
									placeholder='Назва товару'
									className='rounded-2xl border border-white/10 bg-[#071120] px-4 py-3 text-white outline-none'
								/>

								<input
									value={form.slug}
									onChange={e =>
										setForm(prev => ({ ...prev, slug: e.target.value }))
									}
									placeholder='Slug'
									className='rounded-2xl border border-white/10 bg-[#071120] px-4 py-3 text-white outline-none'
								/>

								<input
									value={form.category}
									onChange={e =>
										setForm(prev => ({ ...prev, category: e.target.value }))
									}
									placeholder='Категорія'
									className='rounded-2xl border border-white/10 bg-[#071120] px-4 py-3 text-white outline-none'
								/>

								<input
									type='number'
									value={form.price}
									onChange={e =>
										setForm(prev => ({
											...prev,
											price: Number(e.target.value),
										}))
									}
									placeholder='Ціна'
									className='rounded-2xl border border-white/10 bg-[#071120] px-4 py-3 text-white outline-none'
								/>
							</div>

							<textarea
								value={form.description}
								onChange={e =>
									setForm(prev => ({ ...prev, description: e.target.value }))
								}
								placeholder='Опис товару'
								rows={4}
								className='mt-3 w-full rounded-2xl border border-white/10 bg-[#071120] px-4 py-3 text-white outline-none'
							/>

							<input
								value={form.image_url}
								onChange={e =>
									setForm(prev => ({ ...prev, image_url: e.target.value }))
								}
								placeholder='URL зображення'
								className='mt-3 w-full rounded-2xl border border-white/10 bg-[#071120] px-4 py-3 text-white outline-none'
							/>

							<label className='mt-4 flex items-center gap-3 text-sm text-white/70'>
								<input
									type='checkbox'
									checked={form.is_active}
									onChange={e =>
										setForm(prev => ({
											...prev,
											is_active: e.target.checked,
										}))
									}
								/>
								Товар активний
							</label>

							<div className='mt-4 flex flex-col gap-3 sm:flex-row'>
								<button
									onClick={handleSubmit}
									disabled={saving}
									className='inline-flex items-center justify-center gap-2 rounded-2xl bg-blue-600 px-5 py-3 font-semibold text-white hover:bg-blue-500 disabled:opacity-60'
								>
									<Plus size={18} />
									{form.id ? 'Оновити товар' : 'Додати товар'}
								</button>

								<button
									onClick={resetForm}
									className='rounded-2xl border border-white/10 bg-white/5 px-5 py-3 font-semibold text-white'
								>
									Очистити
								</button>
							</div>

							<div className='mt-6 max-h-[520px] space-y-4 overflow-y-auto pr-2'>
								{products.map(product => (
									<div
										key={product.id}
										className='rounded-2xl border border-white/10 bg-white/5 p-4'
									>
										<div className='flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between'>
											<div className='min-w-0 flex-1'>
												<p className='break-words text-lg font-bold text-white'>
													{product.name}
												</p>
												<p className='mt-1 break-all text-sm text-white/50'>
													/{product.slug} •{' '}
													{product.category || 'Без категорії'}
												</p>
												<p className='mt-2 break-words text-sm leading-relaxed text-white/70'>
													{product.description}
												</p>
											</div>

											<div className='shrink-0 lg:text-right'>
												<p className='text-2xl font-black text-white'>
													{formatPrice(product.price)}
												</p>
												<p
													className={`mt-2 text-xs font-bold uppercase ${
														product.is_active
															? 'text-emerald-400'
															: 'text-red-400'
													}`}
												>
													{product.is_active ? 'Активний' : 'Неактивний'}
												</p>
											</div>
										</div>

										<div className='mt-4 flex flex-col gap-3 sm:flex-row'>
											<button
												onClick={() => startEdit(product)}
												className='rounded-xl border border-blue-500/20 bg-blue-500/10 px-4 py-2 text-sm font-semibold text-blue-200'
											>
												Редагувати
											</button>

											<button
												onClick={() => handleDelete(product.id)}
												className='inline-flex items-center justify-center gap-2 rounded-xl border border-red-500/20 bg-red-500/10 px-4 py-2 text-sm font-semibold text-red-200'
											>
												<Trash2 size={16} />
												Деактивувати
											</button>
										</div>
									</div>
								))}
							</div>
						</section>

						<section className='rounded-[2rem] border border-slate-800 bg-slate-900/40 p-5 shadow-2xl backdrop-blur-md sm:p-6'>
							<div className='mb-6 flex items-center gap-3'>
								<Users className='text-violet-400' />
								<h2 className='text-2xl font-black text-white'>
									Список користувачів
								</h2>
							</div>

							<div className='space-y-3'>
								{users.map(user => {
									const hasValidAvatar =
										user.avatar_url && !brokenAvatars[user.id]

									return (
										<div
											key={user.id}
											className='rounded-2xl border border-white/10 bg-white/5 p-4'
										>
											<div className='flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between'>
												<div className='flex min-w-0 items-center gap-4'>
													<div className='flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-2xl bg-white/10'>
														{hasValidAvatar ? (
															<img
																src={user.avatar_url!}
																alt={user.email}
																className='h-full w-full object-cover'
																onError={() =>
																	setBrokenAvatars(prev => ({
																		...prev,
																		[user.id]: true,
																	}))
																}
															/>
														) : (
															<ShieldCheck
																className='text-white/60'
																size={20}
															/>
														)}
													</div>

													<div className='min-w-0'>
														<p className='break-words font-bold text-white'>
															{user.full_name || user.email.split('@')[0]}
														</p>
														<p className='break-all text-sm text-white/50'>
															{user.email}
														</p>
													</div>
												</div>

												<div className='sm:text-right'>
													<p className='text-sm font-bold uppercase text-white/80'>
														{user.role}
													</p>
													<p className='text-xs text-white/40'>
														{user.auth_provider}
													</p>
												</div>
											</div>
										</div>
									)
								})}
							</div>
						</section>
					</div>

					<section className='flex min-h-[720px] flex-col rounded-[2rem] border border-slate-800 bg-slate-900/40 p-5 shadow-2xl backdrop-blur-md sm:p-6'>
						<div className='mb-5 flex items-center gap-3'>
							<Activity className='text-red-400' />
							<h2 className='text-2xl font-black text-white'>Події системи</h2>
						</div>

						<div className='flex-1 overflow-hidden'>
							<div className='flex h-full max-h-[72vh] flex-col overflow-y-auto pr-1 sm:pr-2'>
								{events.length === 0 ? (
									<div className='flex flex-1 flex-col items-center justify-center rounded-3xl border-2 border-dashed border-slate-800/50 bg-slate-950/20 px-6 py-16 text-center text-slate-600'>
										<ShieldCheck className='mb-5 h-16 w-16 opacity-40' />
										<p className='text-xl font-medium'>
											Очікуємо нові події...
										</p>
										<p className='mt-2 text-sm'>
											Події з’являться тут автоматично.
										</p>
									</div>
								) : (
									<div className='space-y-3'>
										{events.map((event, index) => (
											<div
												key={index}
												className='rounded-2xl border border-white/10 bg-[#071120] p-4'
											>
												<div className='mb-3 flex flex-wrap items-center gap-2'>
													<div className='inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs font-bold uppercase tracking-widest text-white/70'>
														<Clock3 size={12} />
														{event.timestamp}
													</div>

													<div className='inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs font-bold uppercase tracking-widest text-white/70'>
														<Tag size={12} />#{event.order_id}
													</div>

													<div
														className={`inline-flex items-center rounded-full border px-3 py-1 text-xs font-bold uppercase tracking-widest ${getStatusColor(
															event.status,
														)}`}
													>
														{event.status}
													</div>
												</div>

												<p className='break-words text-sm leading-relaxed text-slate-300'>
													{event.message}
												</p>
											</div>
										))}
									</div>
								)}
								<div ref={chatEndRef} />
							</div>
						</div>

						<div className='mt-6 flex flex-col gap-3 border-t border-slate-800 pt-5 text-xs font-bold uppercase tracking-widest text-slate-500 sm:flex-row sm:items-center sm:justify-between'>
							<div className='flex items-center gap-2'>
								<Zap size={14} className='text-red-500' />
								<span>
									Всього подій:{' '}
									<span className='font-mono text-white'>{events.length}</span>
								</span>
							</div>

							<div className='flex items-center gap-2'>
								<AlertTriangle size={14} className='text-amber-500' />
								<span>
									Статус:{' '}
									<span className='font-mono text-white'>
										{isConnected ? 'активно' : "немає з'єднання"}
									</span>
								</span>
							</div>
						</div>
					</section>
				</div>
			</div>
		</div>
	)
}
