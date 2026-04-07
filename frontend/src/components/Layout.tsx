import {
	Activity,
	History,
	LayoutDashboard,
	LogOut,
	User,
	Zap,
} from 'lucide-react'
import React, { useEffect, useState } from 'react'
import { Link, Outlet, useLocation, useNavigate } from 'react-router-dom'
import { authApi } from '../api/auth'
import { Footer } from './Footer'

type AuthUser = {
	id?: number
	user_id?: number
	email?: string
	role?: string
	fullName?: string
	full_name?: string
	avatarUrl?: string | null
	avatar_url?: string | null
}

export const Layout: React.FC = () => {
	const navigate = useNavigate()
	const location = useLocation()

	const [user, setUser] = useState<AuthUser | null>(null)
	const [isAuthenticated, setIsAuthenticated] = useState(false)

	const syncUser = () => {
		const token = authApi.getToken()
		const auth = !!token
		setIsAuthenticated(auth)

		if (!auth) {
			setUser(null)
			return
		}

		try {
			const raw = localStorage.getItem('user')
			if (!raw) {
				setUser(null)
				return
			}

			const parsed = JSON.parse(raw)
			setUser(parsed)
		} catch {
			setUser(null)
		}
	}

	useEffect(() => {
		syncUser()

		window.addEventListener('user-updated', syncUser)
		window.addEventListener('storage', syncUser)

		return () => {
			window.removeEventListener('user-updated', syncUser)
			window.removeEventListener('storage', syncUser)
		}
	}, [])

	useEffect(() => {
		syncUser()
	}, [location.pathname])

	const userId =
		user?.id ??
		user?.user_id ??
		(authApi.getUserId?.() ? Number(authApi.getUserId()) : null)

	const role = (user?.role || authApi.getUserRole?.() || '').toLowerCase()

	const isAdmin = isAuthenticated && role === 'admin'

	const displayName =
		user?.fullName ||
		user?.full_name ||
		user?.email?.split('@')[0] ||
		'User'

	const handleLogout = () => {
		authApi.logout()
		setUser(null)
		setIsAuthenticated(false)
		window.dispatchEvent(new Event('user-updated'))
		navigate('/')
	}

	return (
		<div className='min-h-screen bg-[#030712] flex flex-col'>
			<nav className='sticky top-0 z-50 border-b border-slate-800/60 bg-[#030712]/85 shadow-lg backdrop-blur-xl'>
				<div className='mx-auto flex h-16 max-w-7xl items-center justify-between gap-4 px-4 sm:h-20 sm:px-6'>
					<Link to='/' className='group shrink-0 flex items-center gap-2'>
						<div className='rounded-xl bg-blue-600 p-2 shadow-lg shadow-blue-500/20 transition-transform group-hover:scale-110'>
							<Zap size={18} className='fill-white text-white' />
						</div>
						<span className='text-lg font-black uppercase tracking-tighter text-white sm:text-xl'>
							Pulse<span className='text-blue-600'>Commerce</span>
						</span>
					</Link>

					<div className='flex items-center gap-4 text-[10px] font-bold uppercase tracking-widest sm:gap-8 sm:text-xs'>
						<Link
							to='/'
							className='flex items-center gap-2 text-slate-400 transition hover:text-white'
						>
							<LayoutDashboard size={16} />
							<span className='hidden md:inline'>Вітрина</span>
						</Link>

						{isAuthenticated && (
							<Link
								to='/orders'
								className='flex items-center gap-2 text-slate-400 transition hover:text-white'
							>
								<History size={16} />
								<span className='hidden md:inline'>Історія</span>
							</Link>
						)}

						{isAuthenticated && (
							<Link
								to='/profile'
								className='flex items-center gap-2 text-slate-400 transition hover:text-white'
							>
								<User size={16} />
								<span className='hidden md:inline'>Профіль</span>
							</Link>
						)}

						{isAdmin && (
							<Link
								to='/admin/pulse'
								className='flex items-center gap-2 text-red-500 transition hover:text-red-400'
							>
								<Activity size={16} />
								<span className='hidden md:inline'>Адмінка</span>
							</Link>
						)}
					</div>

					{isAuthenticated ? (
						<div className='flex items-center gap-3 sm:gap-4'>
							<Link
								to='/profile'
								className='hidden text-right transition-opacity hover:opacity-80 sm:block'
							>
								<p className='text-[10px] font-bold uppercase tracking-widest leading-none text-slate-600'>
									{displayName}
								</p>
								<p className='font-black text-white'>#{userId ?? '—'}</p>
							</Link>

							<button
								onClick={handleLogout}
								title='Вийти'
								className='rounded-xl border border-red-500/20 bg-red-500/10 p-2 text-red-500 transition-all hover:bg-red-500 hover:text-white sm:px-4 sm:py-2'
							>
								<LogOut size={16} />
							</button>
						</div>
					) : (
						<div className='flex items-center gap-2 sm:gap-3'>
							<Link
								to='/login'
								className='rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-xs font-bold uppercase tracking-widest text-white transition hover:bg-white/10 sm:text-sm'
							>
								Увійти
							</Link>

							<Link
								to='/register'
								className='rounded-xl bg-blue-600 px-4 py-2 text-xs font-bold uppercase tracking-widest text-white transition hover:bg-blue-500 sm:text-sm'
							>
								Реєстрація
							</Link>
						</div>
					)}
				</div>
			</nav>

			<main className='flex-grow'>
				<Outlet />
			</main>

			<Footer />
		</div>
	)
}