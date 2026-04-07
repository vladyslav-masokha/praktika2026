import { Lock, Mail, Zap } from 'lucide-react'
import React, { useEffect, useState } from 'react'
import { Link, useLocation, useNavigate } from 'react-router-dom'
import { authApi } from '../api/auth'

const API_BASE_URL =
	import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080'

export const LoginPage: React.FC = () => {
	const [email, setEmail] = useState('')
	const [password, setPassword] = useState('')
	const [error, setError] = useState('')
	const [googleLoading, setGoogleLoading] = useState(false)
	const navigate = useNavigate()
	const location = useLocation()

	useEffect(() => {
		const params = new URLSearchParams(location.search)
		const token = params.get('token')
		const errorParam = params.get('error')

		if (errorParam) {
			setError('Не вдалося увійти через Google')
			return
		}

		if (!token) return

		localStorage.setItem('token', token)
		localStorage.setItem('auth_type', 'google')
		window.dispatchEvent(new Event('user-updated'))

		authApi
			.syncProfile()
			.then(profile => {
				if (profile && !localStorage.getItem('display_name')) {
					const fallbackName =
						profile.email?.split('@')[0]?.trim() || 'Користувач'
					localStorage.setItem('display_name', fallbackName)
				}
				navigate('/', { replace: true })
			})
			.catch(() => {
				setError('Не вдалося завершити авторизацію через Google')
			})
	}, [location.search, navigate])

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault()
		setError('')

		try {
			await authApi.login(email, password)
			localStorage.setItem('auth_type', 'password')

			try {
				await authApi.syncProfile()
			} catch {}

			navigate('/')
		} catch {
			setError('Невірний email або пароль')
		}
	}

	const handleGoogleLogin = () => {
		setGoogleLoading(true)
		window.location.href = `${API_BASE_URL}/api/auth/google`
	}

	return (
		<div className='min-h-screen flex items-center justify-center bg-[#030712] px-6 relative overflow-hidden'>
			<div className='absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] bg-blue-600/20 blur-[120px] rounded-full'></div>

			<form
				onSubmit={handleSubmit}
				className='relative bg-gray-900/50 backdrop-blur-xl border border-white/10 p-12 rounded-[2.5rem] shadow-2xl w-full max-w-md'
			>
				<div className='flex justify-center mb-8'>
					<div className='bg-blue-600 p-4 rounded-2xl shadow-lg shadow-blue-600/30'>
						<Zap className='text-white fill-white' size={32} />
					</div>
				</div>

				<h2 className='text-3xl font-black mb-2 uppercase tracking-tighter text-center text-white'>
					Pulse Login
				</h2>

				{error && (
					<p className='text-red-400 text-center text-sm mb-4'>{error}</p>
				)}

				<div className='space-y-5'>
					<div className='relative'>
						<Mail
							className='absolute left-5 top-1/2 -translate-y-1/2 text-gray-500'
							size={20}
						/>
						<input
							type='email'
							value={email}
							onChange={e => setEmail(e.target.value)}
							className='w-full pl-14 pr-6 py-5 bg-black/30 border border-white/5 rounded-2xl text-white outline-none focus:border-blue-600 transition-all'
							placeholder='Email Address'
							required
						/>
					</div>

					<div className='relative'>
						<Lock
							className='absolute left-5 top-1/2 -translate-y-1/2 text-gray-500'
							size={20}
						/>
						<input
							type='password'
							value={password}
							onChange={e => setPassword(e.target.value)}
							className='w-full pl-14 pr-6 py-5 bg-black/30 border border-white/5 rounded-2xl text-white outline-none focus:border-blue-600 transition-all'
							placeholder='Password'
							required
						/>
					</div>

					<button
						type='submit'
						className='w-full bg-blue-600 text-white py-5 rounded-2xl font-black hover:bg-blue-500 transition-all uppercase tracking-widest'
					>
						Увійти
					</button>

					<div className='relative my-6'>
						<div className='absolute inset-0 flex items-center'>
							<div className='w-full border-t border-slate-800/80'></div>
						</div>
						<div className='relative flex justify-center text-[10px] uppercase font-bold'>
							<span className='bg-[#121826] px-4 text-slate-500'>Або</span>
						</div>
					</div>

					<button
						type='button'
						onClick={handleGoogleLogin}
						disabled={googleLoading}
						className='w-full flex items-center justify-center gap-3 p-4 bg-white/5 hover:bg-white/10 rounded-2xl border border-white/10 text-white font-bold transition-all disabled:opacity-70'
					>
						<svg className='w-5 h-5' viewBox='0 0 24 24'>
							<path
								fill='#4285F4'
								d='M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z'
							/>
							<path
								fill='#34A853'
								d='M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z'
							/>
							<path
								fill='#FBBC05'
								d='M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l3.66-2.84z'
							/>
							<path
								fill='#EA4335'
								d='M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z'
							/>
						</svg>
						{googleLoading ? 'Переадресація...' : 'Увійти через Google'}
					</button>

					<p className='text-center text-slate-500 text-sm mt-6'>
						Немає акаунту?{' '}
						<Link to='/register' className='text-blue-500 font-bold'>
							Зареєструватися
						</Link>
					</p>
				</div>
			</form>
		</div>
	)
}
