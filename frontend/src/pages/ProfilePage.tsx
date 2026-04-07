import {
	ChevronRight,
	Home,
	LogOut,
	Mail,
	RefreshCw,
	Save,
	Shield,
	ShoppingBag,
	User,
} from 'lucide-react'
import { useEffect, useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { authApi } from '../api/auth'

type ProfileUser = {
	id: number
	email: string
	role: string
	fullName?: string
	avatarUrl?: string | null
}

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL

if (!API_BASE_URL) {
	throw new Error('VITE_API_BASE_URL is not defined')
}

function normalizeRole(role?: string) {
	const value = (role || '').toLowerCase()

	if (value === 'admin') {
		return {
			label: 'ADMIN',
			description: 'Повний доступ',
			badgeClass: 'border border-red-500/20 bg-red-500/15 text-red-300',
			iconClass: 'text-red-400',
			cardClass: 'border-red-500/20 from-red-500/10 to-red-500/5',
		}
	}

	return {
		label: 'USER',
		description: 'Стандартний доступ',
		badgeClass: 'border border-blue-500/20 bg-blue-500/15 text-blue-300',
		iconClass: 'text-blue-400',
		cardClass: 'border-blue-500/20 from-blue-500/10 to-blue-500/5',
	}
}

function buildAvatarSrc(url?: string | null) {
	const trimmed = url?.trim()

	if (!trimmed) {
		return null
	}

	return trimmed
}

function getStoredUser(): Partial<ProfileUser> | null {
	try {
		const raw = localStorage.getItem('user')

		if (!raw) {
			return null
		}

		const parsed = JSON.parse(raw)

		return {
			id: parsed.id,
			email: parsed.email,
			role: parsed.role,
			fullName: parsed.fullName ?? parsed.full_name ?? '',
			avatarUrl: parsed.avatarUrl ?? parsed.avatar_url ?? null,
		}
	} catch {
		return null
	}
}

function normalizeUser(
	data: any,
	fallback?: Partial<ProfileUser> | null,
): ProfileUser {
	return {
		id: data?.id ?? fallback?.id ?? 0,
		email: data?.email ?? fallback?.email ?? '',
		role: data?.role ?? fallback?.role ?? 'user',
		fullName: data?.fullName ?? data?.full_name ?? fallback?.fullName ?? '',
		avatarUrl:
			data?.avatarUrl ?? data?.avatar_url ?? fallback?.avatarUrl ?? null,
	}
}

export function ProfilePage() {
	const navigate = useNavigate()

	const [user, setUser] = useState<ProfileUser | null>(null)
	const [loading, setLoading] = useState(true)
	const [refreshing, setRefreshing] = useState(false)
	const [saving, setSaving] = useState(false)
	const [error, setError] = useState('')
	const [success, setSuccess] = useState('')
	const [fullName, setFullName] = useState('')
	const [avatarUrl, setAvatarUrl] = useState('')
	const [avatarImageFailed, setAvatarImageFailed] = useState(false)
	const [avatarVersion, setAvatarVersion] = useState(Date.now())

	const roleMeta = normalizeRole(user?.role)
	const previewAvatarSrc = buildAvatarSrc(avatarUrl || user?.avatarUrl)

	const syncUserToStorage = (normalizedUser: ProfileUser) => {
		localStorage.setItem('user', JSON.stringify(normalizedUser))
		localStorage.setItem('user_id', String(normalizedUser.id))
		localStorage.setItem('user_role', normalizedUser.role || 'user')
		window.dispatchEvent(new Event('user-updated'))
	}

	const applyUser = (normalizedUser: ProfileUser) => {
		setUser(normalizedUser)
		setFullName(normalizedUser.fullName || '')
		setAvatarUrl(normalizedUser.avatarUrl || '')
		setAvatarImageFailed(false)
		setAvatarVersion(Date.now())
		syncUserToStorage(normalizedUser)
	}

	const loadProfile = async (silent = false) => {
		const storedUser = getStoredUser()

		try {
			if (!silent) {
				setLoading(true)
			}

			setRefreshing(silent)
			setError('')
			setSuccess('')

			const token = authApi.getToken()

			if (!token) {
				navigate('/login')
				return
			}

			const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
				headers: {
					Authorization: `Bearer ${token}`,
				},
			})

			if (response.status === 401) {
				authApi.logout()
				navigate('/login')
				return
			}

			if (!response.ok) {
				throw new Error('Не вдалося завантажити профіль')
			}

			const data = await response.json()
			const normalizedUser = normalizeUser(data, storedUser)

			applyUser(normalizedUser)
		} catch {
			if (storedUser?.id) {
				applyUser(normalizeUser(storedUser, storedUser))
			} else {
				setError('Не вдалося завантажити актуальні дані профілю')
			}
		} finally {
			setLoading(false)
			setRefreshing(false)
		}
	}

	useEffect(() => {
		loadProfile()
	}, [])

	useEffect(() => {
		setAvatarImageFailed(false)
		setAvatarVersion(Date.now())
	}, [avatarUrl])

	const handleLogout = () => {
		authApi.logout()
		navigate('/login')
	}

	const handleSaveProfile = async () => {
		const token = authApi.getToken()

		if (!token) {
			return
		}

		const trimmedName = fullName.trim()
		const trimmedAvatar = avatarUrl.trim()

		if (!trimmedName) {
			setError("Вкажи ім'я користувача")
			return
		}

		try {
			setSaving(true)
			setError('')
			setSuccess('')

			const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
				method: 'PUT',
				headers: {
					'Content-Type': 'application/json',
					Authorization: `Bearer ${token}`,
				},
				body: JSON.stringify({
					full_name: trimmedName,
					avatar_url: trimmedAvatar || null,
				}),
			})

			if (response.status === 401) {
				authApi.logout()
				navigate('/login')
				return
			}

			if (!response.ok) {
				throw new Error('Не вдалося оновити профіль')
			}

			const data = await response.json()

			const normalizedUser = normalizeUser(data, {
				id: user?.id ?? 0,
				email: user?.email ?? '',
				role: user?.role ?? 'user',
				fullName: trimmedName,
				avatarUrl: trimmedAvatar || null,
			})

			applyUser(normalizedUser)
			setSuccess('Профіль успішно оновлено')
		} catch {
			setError('Помилка при збереженні профілю')
		} finally {
			setSaving(false)
		}
	}

	if (loading) {
		return (
			<div className='flex min-h-screen items-center justify-center bg-[#030817] px-4 text-white'>
				<div className='w-full max-w-md rounded-3xl border border-white/10 bg-white/5 p-8 text-center shadow-2xl backdrop-blur-xl'>
					<div className='mx-auto mb-4 h-12 w-12 animate-spin rounded-full border-2 border-blue-400 border-t-transparent' />
					<p className='text-lg text-white/80'>Завантаження профілю...</p>
				</div>
			</div>
		)
	}

	return (
		<div className='min-h-screen bg-[#030817] text-white'>
			<div className='mx-auto max-w-7xl px-4 py-8 md:px-6 lg:px-8'>
				<div className='mb-8 flex flex-col gap-4 md:flex-row md:items-center md:justify-between'>
					<div>
						<p className='mb-2 text-sm uppercase tracking-[0.25em] text-blue-300/70'>
							Особистий кабінет
						</p>
						<h1 className='text-3xl font-bold md:text-4xl'>
							Профіль користувача
						</h1>
					</div>

					<div className='flex flex-wrap gap-3'>
						<Link
							to='/'
							className='inline-flex items-center gap-2 rounded-2xl border border-white/10 bg-white/5 px-5 py-3 text-sm font-medium text-white/90 transition hover:bg-white/10'
						>
							<Home size={18} />
							На головну
						</Link>

						<button
							onClick={() => loadProfile(true)}
							disabled={refreshing}
							className='inline-flex items-center gap-2 rounded-2xl border border-blue-500/20 bg-blue-500/10 px-5 py-3 text-sm font-medium text-blue-200 transition hover:bg-blue-500/15 disabled:opacity-60'
						>
							<RefreshCw
								size={18}
								className={refreshing ? 'animate-spin' : ''}
							/>
							Оновити профіль
						</button>
					</div>
				</div>

				{error && (
					<div className='mb-6 rounded-2xl border border-red-500/20 bg-red-500/10 px-4 py-3 text-red-200'>
						{error}
					</div>
				)}

				{success && (
					<div className='mb-6 rounded-2xl border border-emerald-500/20 bg-emerald-500/10 px-4 py-3 text-emerald-200'>
						{success}
					</div>
				)}

				<div className='grid gap-6 lg:grid-cols-[380px_minmax(0,1fr)]'>
					<aside className='rounded-[28px] border border-white/10 bg-gradient-to-b from-white/8 to-white/5 p-6 shadow-2xl backdrop-blur-xl'>
						<div className='flex flex-col items-center text-center'>
							<div className='relative mb-5 flex h-28 w-28 items-center justify-center overflow-hidden rounded-[28px] bg-gradient-to-br from-red-500 to-orange-500 shadow-lg shadow-red-900/30'>
								{previewAvatarSrc && !avatarImageFailed ? (
									<img
										key={`${previewAvatarSrc}-${avatarVersion}`}
										src={previewAvatarSrc}
										alt='Avatar'
										className='h-full w-full object-cover'
										referrerPolicy='no-referrer'
										onError={() => setAvatarImageFailed(true)}
									/>
								) : (
									<User size={44} className='text-white' />
								)}
								<span className='absolute bottom-2 right-2 h-5 w-5 rounded-full border-4 border-[#091224] bg-emerald-400' />
							</div>

							<h2 className='text-2xl font-bold'>
								{user?.fullName?.trim() ||
									user?.email?.split('@')[0] ||
									'Користувач'}
							</h2>

							<p className='mt-2 break-all text-sm text-white/60'>
								{user?.email}
							</p>

							<div
								className={`mt-5 inline-flex items-center gap-2 rounded-full px-4 py-2 text-sm font-semibold ${roleMeta.badgeClass}`}
							>
								<Shield size={16} className={roleMeta.iconClass} />
								{roleMeta.label}
							</div>
						</div>

						<div className='mt-8 space-y-3'>
							<Link
								to='/'
								className='flex items-center justify-between rounded-2xl border border-white/8 bg-white/5 px-4 py-4 transition hover:bg-white/10'
							>
								<div className='flex items-center gap-3'>
									<Home className='text-blue-300' size={20} />
									<span className='font-medium'>Головна сторінка</span>
								</div>
								<ChevronRight size={18} className='text-white/40' />
							</Link>

							<Link
								to='/orders'
								className='flex items-center justify-between rounded-2xl border border-white/8 bg-white/5 px-4 py-4 transition hover:bg-white/10'
							>
								<div className='flex items-center gap-3'>
									<ShoppingBag className='text-white/70' size={20} />
									<span className='font-medium'>Мої замовлення</span>
								</div>
								<ChevronRight size={18} className='text-white/40' />
							</Link>
						</div>

						<button
							onClick={handleLogout}
							className='mt-8 flex w-full items-center justify-center gap-2 rounded-2xl bg-gradient-to-r from-red-500/20 to-red-900/20 px-4 py-4 text-lg font-semibold text-red-300 transition hover:from-red-500/30 hover:to-red-900/30'
						>
							<LogOut size={20} />
							Вийти
						</button>
					</aside>

					<section className='rounded-[28px] border border-white/10 bg-gradient-to-b from-[#071327] to-[#050d1e] p-6 shadow-2xl backdrop-blur-xl'>
						<div className='mb-8'>
							<h3 className='text-2xl font-bold'>Налаштування акаунта</h3>
						</div>

						<div className='grid gap-4 md:grid-cols-2'>
							<div className='rounded-[24px] border border-white/10 bg-[#071120] p-5'>
								<div className='mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-blue-500/15'>
									<Mail className='text-blue-300' size={24} />
								</div>
								<p className='text-sm uppercase tracking-wide text-white/40'>
									Email
								</p>
								<p className='mt-2 break-all text-lg font-semibold'>
									{user?.email}
								</p>
							</div>

							<div
								className={`rounded-[24px] border bg-gradient-to-br p-5 ${roleMeta.cardClass}`}
							>
								<div className='mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-white/5'>
									<Shield className={roleMeta.iconClass} size={24} />
								</div>
								<p className='text-sm uppercase tracking-wide text-white/40'>
									Рівень доступу
								</p>
								<p className='mt-2 text-2xl font-bold'>{roleMeta.label}</p>
								<div className='mt-4 inline-flex rounded-full border border-blue-500/20 bg-blue-500/15 px-3 py-1 text-xs font-semibold text-blue-300'>
									{roleMeta.description}
								</div>
							</div>
						</div>

						<div className='mt-8 rounded-[24px] border border-white/10 bg-white/5 p-5'>
							<p className='mb-3 text-sm uppercase tracking-wide text-white/40'>
								Ім&apos;я користувача
							</p>

							<input
								type='text'
								value={fullName}
								onChange={e => setFullName(e.target.value)}
								placeholder="Вкажіть ваше ім'я"
								className='w-full rounded-2xl border border-white/10 bg-[#071120] px-4 py-3 text-white outline-none focus:border-blue-500'
							/>
						</div>

						<div className='mt-6 rounded-[24px] border border-white/10 bg-white/5 p-5'>
							<p className='mb-3 text-sm uppercase tracking-wide text-white/40'>
								URL фото профілю
							</p>

							<input
								type='text'
								value={avatarUrl}
								onChange={e => setAvatarUrl(e.target.value)}
								placeholder='https://...'
								className='w-full rounded-2xl border border-white/10 bg-[#071120] px-4 py-3 text-white outline-none focus:border-blue-500'
							/>
						</div>

						<div className='mt-6 flex justify-end'>
							<button
								onClick={handleSaveProfile}
								disabled={saving}
								className='inline-flex items-center gap-2 rounded-2xl bg-blue-600 px-5 py-3 font-semibold text-white transition hover:bg-blue-500 disabled:opacity-60'
							>
								<Save size={18} />
								{saving ? 'Збереження...' : 'Зберегти профіль'}
							</button>
						</div>
					</section>
				</div>
			</div>
		</div>
	)
}
