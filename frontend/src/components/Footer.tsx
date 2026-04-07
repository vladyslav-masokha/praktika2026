import { Mail, MapPin, Phone, Zap } from 'lucide-react'
import { Link } from 'react-router-dom'

export const Footer = () => {
	return (
		<footer className='border-t border-white/5 bg-[#020611]'>
			<div className='mx-auto max-w-7xl px-4 py-10 sm:px-6 lg:px-8'>
				<div className='grid gap-8 lg:grid-cols-[1.3fr_0.8fr_0.9fr_1fr]'>
					<div>
						<Link to='/' className='group shrink-0 flex items-center gap-2'>
							<div className='rounded-xl bg-blue-600 p-2 shadow-lg shadow-blue-500/20 transition-transform group-hover:scale-110'>
								<Zap size={18} className='fill-white text-white' />
							</div>
							<span className='text-lg font-black uppercase tracking-tighter text-white sm:text-xl'>
								Pulse<span className='text-blue-600'>Commerce</span>
							</span>
						</Link>

						<p className='mt-4 max-w-md text-sm leading-relaxed text-slate-400'>
							Платформа для перегляду товарів, оформлення замовлень та керування
							покупками.
						</p>
					</div>

					<div>
						<h3 className='text-sm font-black uppercase tracking-[0.25em] text-white/80'>
							Навігація
						</h3>
						<div className='mt-4 space-y-3 text-sm'>
							<Link
								to='/'
								className='block text-slate-400 transition hover:text-white'
							>
								Головна
							</Link>
							<Link
								to='/orders'
								className='block text-slate-400 transition hover:text-white'
							>
								Мої замовлення
							</Link>
							<Link
								to='/profile'
								className='block text-slate-400 transition hover:text-white'
							>
								Профіль
							</Link>
							<Link
								to='/admin/pulse'
								className='block text-slate-400 transition hover:text-white'
							>
								Адмінпанель
							</Link>
						</div>
					</div>

					<div>
						<h3 className='text-sm font-black uppercase tracking-[0.25em] text-white/80'>
							Контакти
						</h3>
						<div className='mt-4 space-y-3 text-sm text-slate-400'>
							<div className='flex items-center gap-3'>
								<Mail size={16} className='text-blue-400' />
								<span>pulse_commerce@gmail.com</span>
							</div>
							<div className='flex items-center gap-3'>
								<Phone size={16} className='text-blue-400' />
								<span>+38 (063) 228-07-63</span>
							</div>
							<div className='flex items-center gap-3'>
								<MapPin size={16} className='text-blue-400' />
								<span>Львів, Україна</span>
							</div>
						</div>
					</div>

					<div>
						<h3 className='text-sm font-black uppercase tracking-[0.25em] text-white/80'>
							Інформація
						</h3>
						<p className='mt-4 text-sm leading-relaxed text-slate-400'>
							Оформлюйте замовлення, переглядайте історію покупок і керуйте
							своїм профілем в одному місці.
						</p>
					</div>
				</div>

				<div className='mt-8 flex flex-col gap-3 border-t border-white/5 pt-5 text-xs text-slate-500 sm:flex-row sm:items-center sm:justify-between'>
					<p>&copy;2026. Всі права захищені.</p>
					<p className='inline-flex items-center gap-2'>
						Автори: Масоха В.Ю. та Якименко Д.О.
					</p>
				</div>
			</div>
		</footer>
	)
}
