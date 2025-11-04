import express from 'express';
import cors from 'cors';
import dotenv from 'dotenv';
import compression from 'compression';
import { cacheMiddleware } from './middleware/cache';

dotenv.config();

const app = express();
const port = process.env.PORT || 3001;

// Enable CORS
app.use(cors());

// Enable compression for all responses
app.use(compression());

// Parse JSON bodies
app.use(express.json({ limit: '10mb' }));

// Apply caching middleware to all GET requests (5 minutes cache)
app.use(cacheMiddleware(5 * 60 * 1000));

import locationRouter from './routes/location';
import historyRouter from './routes/history';
import damageRouter from './routes/damage';
import ediRouter from './routes/edi';
import customsRouter from './routes/customs';
import tasksRouter from './routes/tasks';
import appointmentsRouter from './routes/appointments';

// Performance and security headers
app.use((req, res, next) => {
  res.setHeader('X-Content-Type-Options', 'nosniff');
  res.setHeader('X-Frame-Options', 'DENY');
  res.setHeader('X-XSS-Protection', '1; mode=block');
  next();
});

app.get('/', (req, res) => {
  res.json({
    status: 'online',
    message: 'Pistology TOS Backend API',
    version: '1.0.0',
    timestamp: new Date().toISOString()
  });
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    uptime: process.uptime(),
    timestamp: new Date().toISOString()
  });
});

app.use('/api/location', locationRouter);
app.use('/api/history', historyRouter);
app.use('/api/damage', damageRouter);
app.use('/api/edi', ediRouter);
app.use('/api/customs', customsRouter);
app.use('/api/tasks', tasksRouter);
app.use('/api/appointments', appointmentsRouter);

app.listen(port, () => {
  console.log(`Backend server is running on http://localhost:${port}`);
});
