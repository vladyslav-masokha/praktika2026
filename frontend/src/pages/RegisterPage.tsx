import { AlertTriangle, Lock, Mail, UserPlus, Zap } from 'lucide-react'
import React, { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { authApi } from '../api/auth'

const API_BASE_URL =
	import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080'

export const RegisterPage: React.FC = () => {
	const [email, setEmail] = useState('')
	const [password, setPassword] = useState('')
	const [confirmPassword, setConfirmPassword] = useState('')
	const [error, setError] = useState('')
	const [loading, setLoading] = useState(false)
	const [googleLoading, setGoogleLoading] = useState(false)
	const navigate = useNavigate()

	const isAppleDevice = /Mac|iPod|iPhone|iPad/.test(navigator.userAgent)

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault()
		setError('')

		if (password !== confirmPassword) {
			return setError('Паролі не співпадають')
		}

		setLoading(true)
		try {
			await authApi.register(email, password)

			try {
				await authApi.syncProfile()
			} catch {}

			navigate('/')
		} catch (err: any) {
			setError(
				err.message || 'Помилка реєстрації. Можливо, email вже зайнятий.',
			)
		} finally {
			setLoading(false)
		}
	}

	const handleGoogleRegister = () => {
		setGoogleLoading(true)
		window.location.href = `${API_BASE_URL}/api/auth/google`
	}

	return (
		<div className='min-h-screen flex items-center justify-center bg-[#030712] px-6 relative overflow-hidden'>
			<div className='absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-blue-600/10 blur-[120px] rounded-full'></div>

			<form
				onSubmit={handleSubmit}
				className='relative bg-slate-900/50 backdrop-blur-xl border border-slate-800/80 p-10 sm:p-12 rounded-[2.5rem] shadow-2xl w-full max-w-md'
			>
				<div className='flex justify-center mb-6'>
					<div className='bg-blue-600 p-4 rounded-2xl shadow-lg shadow-blue-600/30'>
						<Zap className='text-white fill-white' size={28} />
					</div>
				</div>

				<h2 className='text-3xl font-black mb-2 uppercase tracking-tighter text-center text-white'>
					Реєстрація
				</h2>

				{error && (
					<div className='flex items-center gap-2.5 p-4 mb-6 bg-red-500/5 border border-red-500/20 rounded-xl text-red-400 text-sm font-medium animate-in slide-in-from-top-2'>
						<AlertTriangle size={18} /> {error}
					</div>
				)}

				<div className='space-y-4'>
					<div className='relative'>
						<Mail
							className='absolute left-5 top-1/2 -translate-y-1/2 text-slate-500'
							size={20}
						/>
						<input
							type='email'
							value={email}
							onChange={e => setEmail(e.target.value)}
							required
							className='w-full pl-14 pr-6 py-4 bg-slate-950/60 border border-slate-800/80 rounded-2xl text-white placeholder:text-slate-600 focus:border-blue-600 outline-none transition-all'
							placeholder='Email адреса'
						/>
					</div>

					<div className='relative'>
						<Lock
							className='absolute left-5 top-1/2 -translate-y-1/2 text-slate-500'
							size={20}
						/>
						<input
							type='password'
							value={password}
							onChange={e => setPassword(e.target.value)}
							required
							className='w-full pl-14 pr-6 py-4 bg-slate-950/60 border border-slate-800/80 rounded-2xl text-white placeholder:text-slate-600 focus:border-blue-600 outline-none transition-all'
							placeholder='Пароль'
						/>
					</div>

					<div className='relative'>
						<Lock
							className='absolute left-5 top-1/2 -translate-y-1/2 text-slate-500'
							size={20}
						/>
						<input
							type='password'
							value={confirmPassword}
							onChange={e => setConfirmPassword(e.target.value)}
							required
							className='w-full pl-14 pr-6 py-4 bg-slate-950/60 border border-slate-800/80 rounded-2xl text-white placeholder:text-slate-600 focus:border-blue-600 outline-none transition-all'
							placeholder='Підтвердіть пароль'
						/>
					</div>

					<button
						type='submit'
						disabled={loading}
						className='w-full flex items-center justify-center gap-2 mt-2 bg-blue-600 text-white py-4 rounded-2xl font-black hover:bg-blue-500 transition-all uppercase tracking-widest disabled:opacity-70'
					>
						{loading ? (
							'Створення...'
						) : (
							<>
								<UserPlus size={18} /> Зареєструватися
							</>
						)}
					</button>

					<div className='relative my-6'>
						<div className='absolute inset-0 flex items-center'>
							<div className='w-full border-t border-slate-800/80'></div>
						</div>
						<div className='relative flex justify-center text-[10px] uppercase tracking-widest font-bold'>
							<span className='bg-[#0b1120] px-4 text-slate-500 rounded-full'>
								Або
							</span>
						</div>
					</div>

					<div className='space-y-3'>
						<button
							type='button'
							onClick={handleGoogleRegister}
							disabled={googleLoading}
							className='w-full flex items-center justify-center gap-3 p-4 bg-white/5 hover:bg-white/10 rounded-2xl border border-white/10 transition-colors text-sm font-bold text-white active:scale-[0.98] disabled:opacity-70'
						>
							<svg className='w-5 h-5' viewBox='0 0 24 24'>
								<path
									fill='currentColor'
									d='M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z'
								/>
								<path
									fill='#34A853'
									d='M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z'
								/>
								<path
									fill='#FBBC05'
									d='M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z'
								/>
								<path
									fill='#EA4335'
									d='M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z'
								/>
							</svg>
							{googleLoading ? 'Переадресація...' : 'Через Google'}
						</button>

						{isAppleDevice && (
							<button
								type='button'
								className='w-full flex items-center justify-center gap-3 p-4 bg-white text-black hover:bg-slate-200 rounded-2xl transition-colors text-sm font-bold active:scale-[0.98]'
							>
								<svg
									className='w-5 h-5'
									viewBox='0 0 24 24'
									fill='currentColor'
								>
									<path d='M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.15 2.87.8 3.68 2.05-3.14 1.87-2.64 5.9.46 7.15-.71 1.69-1.55 3.23-2.79 3.81zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.32 2.34-1.92 4.35-3.74 4.25z' />
								</svg>
								Через Apple
							</button>
						)}
					</div>
				</div>

				<div className='mt-8 text-center'>
					<p className='text-slate-500 text-sm font-medium'>
						Вже маєте акаунт?{' '}
						<Link
							to='/login'
							className='text-blue-500 hover:text-blue-400 font-bold transition-colors'
						>
							Увійти
						</Link>
					</p>
				</div>
			</form>
		</div>
	)
}
