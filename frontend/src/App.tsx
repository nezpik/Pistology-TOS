import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { lazy, Suspense } from 'react';
import Sidebar from './components/Sidebar';
import './index.css';

// Lazy load all page components for better performance
const ContainerLocation = lazy(() => import('./pages/ContainerLocation'));
const ContainerHistory = lazy(() => import('./pages/ContainerHistory'));
const DamageReport = lazy(() => import('./pages/DamageReport'));
const EdiHub = lazy(() => import('./pages/EdiHub'));
const CustomsInspection = lazy(() => import('./pages/CustomsInspection'));
const TaskDashboard = lazy(() => import('./pages/TaskDashboard'));
const TruckAppointmentSystem = lazy(() => import('./pages/TruckAppointmentSystem'));
const TruckingCompanyView = lazy(() => import('./pages/TruckingCompanyView'));

// Loading component
const LoadingFallback = () => (
  <div className="flex items-center justify-center h-full">
    <div className="text-center">
      <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
      <p className="mt-4 text-gray-600">Loading...</p>
    </div>
  </div>
);

function App() {
  return (
    <Router>
      <div className="App flex h-screen bg-secondary-100 text-gray-800">
        <Sidebar />
        <main className="flex-1 p-8 overflow-y-auto">
          <Suspense fallback={<LoadingFallback />}>
            <Routes>
              <Route path="/" element={<TaskDashboard />} />
              <Route path="/location/:containerId" element={<ContainerLocation />} />
              <Route path="/history/:containerId" element={<ContainerHistory />} />
              <Route path="/damage/:containerId" element={<DamageReport />} />
              <Route path="/edi/:containerId" element={<EdiHub />} />
              <Route path="/customs/:containerId" element={<CustomsInspection />} />
              <Route path="/tasks" element={<TaskDashboard />} />
              <Route path="/tas" element={<TruckAppointmentSystem />} />
              <Route path="/tas/:companyName" element={<TruckingCompanyView />} />
            </Routes>
          </Suspense>
        </main>
      </div>
    </Router>
  );
}

export default App;
