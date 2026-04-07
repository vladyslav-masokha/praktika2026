import {
	ArrowLeft,
	CheckCircle2,
	CreditCard,
	ImageOff,
	Loader2,
	ShoppingCart,
	Truck,
} from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { authApi } from '../api/auth'
import { LiveCheckoutModal } from '../components/LiveCheckoutModal'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL

if (!API_BASE_URL) {
	throw new Error('VITE_API_BASE_URL is not defined')
}

type Product = {
	id: number
	slug: string
	name: string
	description: string
	price: number
	image_url?: string | null
	category?: string | null
}

type CreateOrderResponse = {
	id: number
	user_id: number
	product_id?: number | null
	product_slug?: string | null
	product_name?: string | null
	product_image_url?: string | null
	amount: number
	status: string
	created_at: string
}

type AuthRequiredModalProps = {
	open: boolean
	onClose: () => void
	onLogin: () => void
	onRegister: () => void
}

const fallbackGradient = [
	'from-blue-600/30 to-cyan-500/20',
	'from-purple-600/30 to-fuchsia-500/20',
	'from-emerald-600/30 to-teal-500/20',
	'from-orange-600/30 to-amber-500/20',
]

const AuthRequiredModal = ({
	open,
	onClose,
	onLogin,
	onRegister,
}: AuthRequiredModalProps) => {
	if (!open) {
		return null
	}

	return (
		<div className='fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4 backdrop-blur-sm'>
			<div className='w-full max-w-md rounded-[2rem] border border-white/10 bg-[#07111f] p-6 shadow-2xl sm:p-8'>
				<h2 className='text-2xl font-black tracking-tight text-white sm:text-3xl'>
					Потрібна авторизація
				</h2>
				<p className='mt-3 text-sm leading-relaxed text-slate-400 sm:text-base'>
					Щоб купити товар, потрібно увійти в акаунт або створити новий.
				</p>

				<div className='mt-6 space-y-3'>
					<button
						onClick={onLogin}
						className='w-full rounded-2xl bg-blue-600 px-5 py-3.5 font-black uppercase tracking-widest text-white transition hover:bg-blue-500'
					>
						Увійти
					</button>

					<button
						onClick={onRegister}
						className='w-full rounded-2xl border border-white/10 bg-white/5 px-5 py-3.5 font-black uppercase tracking-widest text-white transition hover:bg-white/10'
					>
						Реєстрація
					</button>

					<button
						onClick={onClose}
						className='w-full rounded-2xl border border-red-500/20 bg-red-500/10 px-5 py-3.5 font-black uppercase tracking-widest text-red-300 transition hover:bg-red-500/15'
					>
						Закрити
					</button>
				</div>
			</div>
		</div>
	)
}

function formatPrice(value: number) {
	return `${value.toLocaleString('uk-UA')} ₴`
}

function getProductImage(product: Product | null) {
	if (product?.image_url && product.image_url.trim().length > 0) {
		return product.image_url.trim()
	}

	return null
}

async function fetchProducts(): Promise<Product[]> {
	const response = await fetch(`${API_BASE_URL}/api/products`)

	if (!response.ok) {
		return []
	}

	const data = await response.json()
	return Array.isArray(data) ? data : []
}

async function createOrder(payload: {
	product_id: number
	product_slug: string
	product_name: string
	product_image_url?: string | null
	amount: number
}): Promise<CreateOrderResponse> {
	const token = authApi.getToken()

	if (!token) {
		throw new Error('Необхідна авторизація')
	}

	const response = await fetch(`${API_BASE_URL}/api/orders`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${token}`,
		},
		body: JSON.stringify(payload),
	})

	if (response.status === 401) {
		authApi.logout()
		throw new Error('Сесія завершилася. Увійдіть знову.')
	}

	if (!response.ok) {
		const text = await response.text().catch(() => '')
		throw new Error(text || 'Не вдалося створити замовлення')
	}

	return response.json()
}

export const ProductDetailsPage = () => {
	const { slug } = useParams()
	const navigate = useNavigate()

	const [products, setProducts] = useState<Product[]>([])
	const [loading, setLoading] = useState(true)
	const [imageError, setImageError] = useState(false)
	const [authModalOpen, setAuthModalOpen] = useState(false)
	const [isAuthenticated, setIsAuthenticated] = useState(
		authApi.isAuthenticated(),
	)
	const [isProcessing, setIsProcessing] = useState(false)
	const [activeOrderId, setActiveOrderId] = useState<number | null>(null)
	const [buyError, setBuyError] = useState('')

	useEffect(() => {
		const syncAuth = () => {
			setIsAuthenticated(authApi.isAuthenticated())
		}

		syncAuth()
		window.addEventListener('user-updated', syncAuth)
		window.addEventListener('storage', syncAuth)

		return () => {
			window.removeEventListener('user-updated', syncAuth)
			window.removeEventListener('storage', syncAuth)
		}
	}, [])

	useEffect(() => {
		const loadProducts = async () => {
			try {
				setLoading(true)
				const data = await fetchProducts()
				setProducts(data)
			} finally {
				setLoading(false)
			}
		}

		loadProducts()
	}, [])

	const product = useMemo(
		() => products.find(item => item.slug === slug) || null,
		[products, slug],
	)

	const relatedProducts = useMemo(() => {
		if (!product) {
			return []
		}

		return products
			.filter(item => item.id !== product.id)
			.filter(item =>
				product.category ? item.category === product.category : true,
			)
			.slice(0, 3)
	}, [products, product])

	const handleBuy = async () => {
		if (!product) {
			return
		}

		setBuyError('')

		if (!authApi.isAuthenticated()) {
			setAuthModalOpen(true)
			return
		}

		try {
			setIsProcessing(true)

			const order = await createOrder({
				product_id: product.id,
				product_slug: product.slug,
				product_name: product.name,
				product_image_url: product.image_url ?? null,
				amount: product.price,
			})

			setActiveOrderId(order.id)
		} catch (err) {
			setBuyError(
				err instanceof Error ? err.message : 'Не вдалося оформити покупку',
			)
		} finally {
			setIsProcessing(false)
		}
	}

	if (loading) {
		return (
			<div className='min-h-screen bg-[#030712] px-4 py-16 text-white'>
				<div className='mx-auto max-w-7xl'>
					<div className='grid gap-8 lg:grid-cols-[1.05fr_0.95fr]'>
						<div className='aspect-square animate-pulse rounded-[2rem] bg-slate-900/40' />
						<div className='space-y-4'>
							<div className='h-6 w-28 animate-pulse rounded bg-slate-900/40' />
							<div className='h-12 w-3/4 animate-pulse rounded bg-slate-900/40' />
							<div className='h-4 w-full animate-pulse rounded bg-slate-900/30' />
							<div className='h-4 w-5/6 animate-pulse rounded bg-slate-900/30' />
							<div className='h-28 animate-pulse rounded-[2rem] bg-slate-900/35' />
						</div>
					</div>
				</div>
			</div>
		)
	}

	if (!product) {
		return (
			<div className='min-h-screen bg-[#030712] px-4 py-16 text-white'>
				<div className='mx-auto max-w-3xl rounded-[2rem] border border-white/5 bg-slate-900/30 p-8 text-center sm:p-12'>
					<h1 className='text-3xl font-black text-white sm:text-4xl'>
						Товар не знайдено
					</h1>
					<p className='mt-4 text-slate-400'>
						Можливо, цей товар був видалений або його адреса змінилася.
					</p>

					<Link
						to='/'
						className='mt-8 inline-flex items-center gap-2 rounded-2xl bg-blue-600 px-5 py-3 font-black uppercase tracking-widest text-white transition hover:bg-blue-500'
					>
						<ArrowLeft size={16} />
						Повернутися до каталогу
					</Link>
				</div>
			</div>
		)
	}

	const image = getProductImage(product)
	const gradient =
		fallbackGradient[product.id % fallbackGradient.length] ||
		fallbackGradient[0]

	return (
		<div className='min-h-screen bg-[#030712] text-white'>
			<section className='border-b border-white/5 px-4 py-8 sm:px-6 lg:px-8'>
				<div className='mx-auto max-w-7xl'>
					<Link
						to='/'
						className='inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-2 text-xs font-black uppercase tracking-widest text-white transition hover:bg-white/10'
					>
						<ArrowLeft size={14} />
						До каталогу
					</Link>
				</div>
			</section>

			<main className='mx-auto max-w-7xl px-4 py-10 sm:px-6 lg:px-8'>
				{buyError && (
					<div className='mb-6 rounded-2xl border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-200'>
						{buyError}
					</div>
				)}

				<div className='grid gap-8 lg:grid-cols-[1.05fr_0.95fr]'>
					<div className='overflow-hidden rounded-[2rem] border border-white/5 bg-slate-900/30'>
						<div className='relative aspect-square overflow-hidden bg-[#071120]'>
							{image && !imageError ? (
								<img
									src={image}
									alt={product.name}
									className='h-full w-full object-cover'
									onError={() => setImageError(true)}
								/>
							) : (
								<div
									className={`flex h-full w-full items-center justify-center bg-gradient-to-br ${gradient}`}
								>
									<div className='rounded-[1.5rem] border border-white/10 bg-black/20 px-6 py-5 text-center backdrop-blur-sm'>
										<div className='mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-2xl border border-white/10 bg-white/10'>
											<ImageOff size={24} className='text-white/80' />
										</div>
										<p className='mt-2 text-2xl font-black text-white'>
											{product.name}
										</p>
									</div>
								</div>
							)}
						</div>
					</div>

					<div className='flex flex-col'>
						<div className='rounded-[2rem] border border-white/5 bg-slate-900/30 p-6 sm:p-8'>
							<div className='flex flex-wrap items-center gap-3'>
								{product.category && (
									<div className='rounded-full border border-blue-500/20 bg-blue-500/10 px-3 py-1.5 text-[10px] font-black uppercase tracking-[0.25em] text-blue-300'>
										{product.category}
									</div>
								)}

								<div className='rounded-full border border-emerald-500/15 bg-emerald-500/10 px-3 py-1.5 text-[10px] font-black uppercase tracking-[0.25em] text-emerald-300'>
									В наявності
								</div>
							</div>

							<h1 className='mt-5 text-4xl font-black leading-tight tracking-tight text-white sm:text-5xl'>
								{product.name}
							</h1>

							<p className='mt-5 text-base leading-relaxed text-slate-400'>
								{product.description}
							</p>

							<div className='mt-8 rounded-[1.75rem] border border-white/5 bg-[#071120] p-5 sm:p-6'>
								<p className='text-[10px] font-black uppercase tracking-[0.25em] text-slate-500'>
									Ціна
								</p>
								<p className='mt-2 text-4xl font-black text-white sm:text-5xl'>
									{formatPrice(product.price)}
								</p>

								<div className='mt-5 grid gap-3 sm:grid-cols-2'>
									<div className='rounded-2xl border border-white/5 bg-white/5 p-4'>
										<div className='flex items-center gap-3'>
											<CreditCard className='text-blue-400' size={20} />
											<div>
												<p className='text-sm font-bold text-white'>
													Оплата онлайн
												</p>
												<p className='text-xs text-slate-500'>
													Підтвердження після оформлення замовлення
												</p>
											</div>
										</div>
									</div>

									<div className='rounded-2xl border border-white/5 bg-white/5 p-4'>
										<div className='flex items-center gap-3'>
											<Truck className='text-blue-400' size={20} />
											<div>
												<p className='text-sm font-bold text-white'>Доставка</p>
												<p className='text-xs text-slate-500'>
													Інформація про замовлення доступна у кабінеті
												</p>
											</div>
										</div>
									</div>
								</div>

								<div className='mt-6 grid gap-3 sm:grid-cols-2'>
									<button
										onClick={handleBuy}
										disabled={isProcessing}
										className='inline-flex items-center justify-center gap-2 rounded-2xl bg-blue-600 px-5 py-4 font-black uppercase tracking-widest text-white transition hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-70'
									>
										{isProcessing ? (
											<>
												<Loader2 size={18} className='animate-spin' />
												Обробка
											</>
										) : (
											<>
												<ShoppingCart size={18} />
												{isAuthenticated
													? 'Купити зараз'
													: 'Увійти для покупки'}
											</>
										)}
									</button>

									<Link
										to='/orders'
										className='inline-flex items-center justify-center gap-2 rounded-2xl border border-white/10 bg-white/5 px-5 py-4 font-black uppercase tracking-widest text-white transition hover:bg-white/10'
									>
										<CheckCircle2 size={18} />
										Мої замовлення
									</Link>
								</div>
							</div>
						</div>
					</div>
				</div>

				<section className='mt-12'>
					<div className='mb-6 flex items-center justify-between gap-4'>
						<h2 className='text-2xl font-black uppercase tracking-tight text-white'>
							Схожі товари
						</h2>
						<Link
							to='/'
							className='text-sm font-bold uppercase tracking-widest text-blue-300 transition hover:text-blue-200'
						>
							Увесь каталог
						</Link>
					</div>

					{relatedProducts.length === 0 ? (
						<div className='rounded-[2rem] border border-white/5 bg-slate-900/20 px-6 py-10 text-center text-slate-500'>
							Схожих товарів поки немає.
						</div>
					) : (
						<div className='grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3'>
							{relatedProducts.map((item, index) => {
								const relatedImage = getProductImage(item)
								const relatedGradient =
									fallbackGradient[index % fallbackGradient.length]

								return (
									<Link
										key={item.id}
										to={`/products/${item.slug}`}
										className='group overflow-hidden rounded-[2rem] border border-white/5 bg-slate-900/30 transition hover:border-blue-500/20 hover:bg-slate-900/50'
									>
										<div className='relative aspect-[4/3] overflow-hidden bg-[#071120]'>
											{relatedImage ? (
												<img
													src={relatedImage}
													alt={item.name}
													className='h-full w-full object-cover transition-transform duration-500 group-hover:scale-105'
												/>
											) : (
												<div
													className={`flex h-full w-full items-center justify-center bg-gradient-to-br ${relatedGradient}`}
												>
													<div className='rounded-[1.5rem] border border-white/10 bg-black/20 px-5 py-4 text-center backdrop-blur-sm'>
														<p className='text-lg font-black text-white'>
															{item.name}
														</p>
													</div>
												</div>
											)}
											<div className='absolute inset-0 bg-gradient-to-t from-[#030712]/80 via-transparent to-transparent' />
										</div>

										<div className='p-5'>
											<p className='text-xl font-black text-white'>
												{item.name}
											</p>
											<p className='mt-2 line-clamp-2 text-sm text-slate-400'>
												{item.description}
											</p>
											<p className='mt-4 text-2xl font-black text-white'>
												{formatPrice(item.price)}
											</p>
										</div>
									</Link>
								)
							})}
						</div>
					)}
				</section>
			</main>

			<AuthRequiredModal
				open={authModalOpen}
				onClose={() => setAuthModalOpen(false)}
				onLogin={() => navigate('/login')}
				onRegister={() => navigate('/register')}
			/>

			<LiveCheckoutModal
				orderId={activeOrderId}
				onClose={() => setActiveOrderId(null)}
			/>
		</div>
	)
}
