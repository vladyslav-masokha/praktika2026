import { ArrowRight, ImageOff, Loader2, ShoppingCart } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
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
				<div className='mb-5'>
					<h2 className='text-2xl font-black tracking-tight text-white sm:text-3xl'>
						Потрібна авторизація
					</h2>
					<p className='mt-3 text-sm leading-relaxed text-slate-400 sm:text-base'>
						Для оформлення замовлення потрібно увійти в акаунт або створити
						новий.
					</p>
				</div>

				<div className='space-y-3'>
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

function getProductImage(product: Product) {
	if (product.image_url && product.image_url.trim().length > 0) {
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

export const HomePage = () => {
	const navigate = useNavigate()

	const [products, setProducts] = useState<Product[]>([])
	const [isLoading, setIsLoading] = useState(true)
	const [authModalOpen, setAuthModalOpen] = useState(false)
	const [isAuthenticated, setIsAuthenticated] = useState(
		authApi.isAuthenticated(),
	)
	const [isProcessingId, setIsProcessingId] = useState<number | null>(null)
	const [activeOrderId, setActiveOrderId] = useState<number | null>(null)
	const [buyError, setBuyError] = useState('')
	const [imageErrors, setImageErrors] = useState<Record<number, boolean>>({})

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
				setIsLoading(true)
				const data = await fetchProducts()
				setProducts(data)
			} finally {
				setIsLoading(false)
			}
		}

		loadProducts()
	}, [])

	const handleBuy = async (product: Product) => {
		setBuyError('')

		if (!authApi.isAuthenticated()) {
			setAuthModalOpen(true)
			return
		}

		try {
			setIsProcessingId(product.id)

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
			setIsProcessingId(null)
		}
	}

	return (
		<div className='min-h-screen bg-[#030712] text-white'>
			<section className='relative overflow-hidden border-b border-white/5 px-4 py-14 sm:px-8 sm:py-24'>
				<div className='absolute inset-0 bg-[radial-gradient(circle_at_top,rgba(37,99,235,0.22),transparent_40%),radial-gradient(circle_at_bottom_right,rgba(14,165,233,0.12),transparent_28%)]' />
				<div className='relative mx-auto max-w-7xl'>
					<div className='max-w-3xl'>
						<h1 className='text-4xl font-black uppercase tracking-tighter text-white sm:text-6xl'>
							Техніка для роботи
							<span className='text-blue-500'> та щоденного життя</span>
						</h1>

						<p className='mt-5 max-w-2xl text-sm font-medium leading-relaxed text-slate-400 sm:text-base'>
							Обирай сучасні товари, переглядай деталі та оформлюй замовлення в
							кілька кліків. Каталог адаптований під мобільні пристрої та
							зручний для швидкого перегляду.
						</p>

						{!isAuthenticated && (
							<div className='mt-7 inline-flex rounded-full border border-blue-500/20 bg-blue-500/10 px-4 py-2 text-xs font-bold uppercase tracking-widest text-blue-300'>
								Для покупки потрібен вхід
							</div>
						)}
					</div>
				</div>
			</section>

			<main className='mx-auto w-full max-w-7xl px-4 py-10 sm:px-6 sm:py-16'>
				<div className='mb-10'>
					<div className='mb-3 h-1 w-16 rounded-full bg-blue-600' />
					<h2 className='text-2xl font-black uppercase tracking-tight text-white sm:text-3xl'>
						Наші товари
					</h2>
					<p className='mt-2 text-sm font-medium text-slate-500'>
						Оберіть товар, перегляньте деталі та оформіть покупку
					</p>
				</div>

				{buyError && (
					<div className='mb-6 rounded-2xl border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-200'>
						{buyError}
					</div>
				)}

				{isLoading ? (
					<div className='grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3'>
						{Array.from({ length: 6 }).map((_, index) => (
							<div
								key={index}
								className='overflow-hidden rounded-[2rem] border border-white/5 bg-slate-900/30'
							>
								<div className='aspect-[4/3] animate-pulse bg-slate-800/60' />
								<div className='space-y-4 p-6'>
									<div className='h-6 w-3/4 animate-pulse rounded bg-slate-800/60' />
									<div className='h-4 w-full animate-pulse rounded bg-slate-800/40' />
									<div className='h-4 w-5/6 animate-pulse rounded bg-slate-800/40' />
									<div className='h-12 w-full animate-pulse rounded-2xl bg-slate-800/50' />
								</div>
							</div>
						))}
					</div>
				) : products.length === 0 ? (
					<div className='rounded-[2.5rem] border border-white/8 bg-slate-900/20 px-6 py-20 text-center'>
						<p className='text-2xl font-black text-white'>
							Поки що товарів немає
						</p>
						<p className='mt-3 text-sm font-medium text-slate-500'>
							Зараз каталог оновлюється. Завітайте трохи пізніше.
						</p>
					</div>
				) : (
					<div className='grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3'>
						{products.map((product, index) => {
							const image = getProductImage(product)
							const gradient = fallbackGradient[index % fallbackGradient.length]
							const imageBroken = imageErrors[product.id]

							return (
								<article
									key={product.id}
									className='group flex h-full flex-col overflow-hidden rounded-[2rem] border border-white/5 bg-slate-900/30 transition-all duration-300 hover:border-blue-500/20 hover:bg-slate-900/50'
								>
									<div className='relative aspect-[4/3] overflow-hidden bg-[#071120]'>
										{image && !imageBroken ? (
											<img
												src={image}
												alt={product.name}
												className='h-full w-full object-cover transition-transform duration-500 group-hover:scale-105'
												onError={() =>
													setImageErrors(prev => ({
														...prev,
														[product.id]: true,
													}))
												}
											/>
										) : (
											<div
												className={`flex h-full w-full items-center justify-center bg-gradient-to-br ${gradient}`}
											>
												<div className='mx-4 rounded-[1.5rem] border border-white/10 bg-black/20 px-5 py-4 text-center backdrop-blur-sm'>
													<div className='mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-2xl border border-white/10 bg-white/10'>
														<ImageOff size={22} className='text-white/80' />
													</div>
													<p className='mt-2 text-lg font-black text-white'>
														{product.name}
													</p>
												</div>
											</div>
										)}

										<div className='absolute inset-0 bg-gradient-to-t from-[#030712]/80 via-transparent to-transparent' />

										{product.category && (
											<div className='absolute left-4 top-4 max-w-[70%] truncate rounded-full border border-white/10 bg-black/40 px-3 py-1.5 text-[10px] font-black uppercase tracking-[0.2em] text-white backdrop-blur-sm'>
												{product.category}
											</div>
										)}
									</div>

									<div className='flex flex-1 flex-col p-5 sm:p-6'>
										<h3 className='text-2xl font-black leading-tight text-white'>
											{product.name}
										</h3>

										<p className='mt-3 min-h-[72px] text-sm font-medium leading-relaxed text-slate-400'>
											{product.description}
										</p>

										<div className='mt-6 border-t border-white/5 pt-5'>
											<p className='text-[10px] font-black uppercase tracking-[0.25em] text-slate-600'>
												Ціна
											</p>
											<p className='mt-2 text-3xl font-black text-white'>
												{formatPrice(product.price)}
											</p>
										</div>

										<div className='mt-6 grid grid-cols-1 gap-3 sm:grid-cols-2'>
											<Link
												to={`/products/${product.slug}`}
												className='inline-flex items-center justify-center gap-2 rounded-2xl border border-white/10 bg-white/5 px-4 py-3 font-black uppercase tracking-widest text-white transition hover:bg-white/10'
											>
												Детальніше
												<ArrowRight size={16} />
											</Link>

											<button
												onClick={() => handleBuy(product)}
												disabled={isProcessingId === product.id}
												className='inline-flex items-center justify-center gap-2 rounded-2xl bg-blue-600 px-4 py-3 font-black uppercase tracking-widest text-white transition hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-70'
											>
												{isProcessingId === product.id ? (
													<>
														<Loader2 size={16} className='animate-spin' />
														Обробка
													</>
												) : (
													<>
														<ShoppingCart size={16} />
														{isAuthenticated ? 'Купити' : 'Увійти'}
													</>
												)}
											</button>
										</div>
									</div>
								</article>
							)
						})}
					</div>
				)}
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
