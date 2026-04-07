import React from 'react'
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { Layout } from './components/Layout'
import { AdminPulsePage } from './pages/AdminPulsePage'
import { HomePage } from './pages/HomePage'
import { LoginPage } from './pages/LoginPage'
import { OrdersPage } from './pages/OrdersPage'
import { ProductDetailsPage } from './pages/ProductDetailsPage'
import { ProfilePage } from './pages/ProfilePage'
import { RegisterPage } from './pages/RegisterPage'

const ProtectedRoute = ({ children }: { children: React.ReactNode }) => {
	const token = localStorage.getItem('token')
	if (!token) return <Navigate to='/login' replace />
	return <>{children}</>
}

const AdminRoute = ({ children }: { children: React.ReactNode }) => {
	const token = localStorage.getItem('token')

	let userRole = ''
	try {
		const raw = localStorage.getItem('user')
		if (raw) {
			const parsed = JSON.parse(raw)
			userRole = (parsed?.role || '').toLowerCase()
		}
	} catch {
		userRole = ''
	}

	if (!token) return <Navigate to='/login' replace />

	if (userRole !== 'admin') {
		return <Navigate to='/' replace />
	}

	return <>{children}</>
}

const App: React.FC = () => {
	return (
		<BrowserRouter>
			<Routes>
				<Route path='/login' element={<LoginPage />} />
				<Route path='/register' element={<RegisterPage />} />

				<Route path='/' element={<Layout />}>
					<Route index element={<HomePage />} />
					<Route path='products/:slug' element={<ProductDetailsPage />} />

					<Route
						path='orders'
						element={
							<ProtectedRoute>
								<OrdersPage />
							</ProtectedRoute>
						}
					/>

					<Route
						path='profile'
						element={
							<ProtectedRoute>
								<ProfilePage />
							</ProtectedRoute>
						}
					/>

					<Route
						path='admin/pulse'
						element={
							<AdminRoute>
								<AdminPulsePage />
							</AdminRoute>
						}
					/>
				</Route>

				<Route path='*' element={<Navigate to='/' replace />} />
			</Routes>
		</BrowserRouter>
	)
}

export default App
