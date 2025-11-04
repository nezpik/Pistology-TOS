# Pistology Terminal Operating System

A modern, high-performance web-based terminal operating system for managing container logistics. Built with React, Node.js, and SQLite, fully containerized with Docker.

## Features

- **System of container localization (GTC)** - Real-time GPS tracking
- **Digital container structure display** - Bay, row, tier visualization
- **Ship structure integration via BAPLIE** - EDI message parsing
- **EDI Hub** - Support for BAPLIE, COARRI, and CODECO messages
- **Digital Container Damage Report (CDR)** - Photo documentation
- **Digital customs inspection** - Workflow management
- **Task tracking dashboard** - Team collaboration
- **Container history interface** - Event timeline
- **Truck Appointment System (TAS)** - Dual interface for operators and trucking companies

## Performance Optimizations

- **SQLite database** with comprehensive indexing
- **In-memory caching** with automatic expiration
- **Response compression** (gzip)
- **Frontend code splitting** and lazy loading
- **Build optimizations** with Terser minification
- **Health checks** for monitoring
- **Security headers** enabled

## Prerequisites

### Option 1: Docker (Recommended)
- Docker Engine 20.10+
- Docker Compose 2.0+

### Option 2: Local Development
- Node.js (v18 or later)
- npm 8+

## Quick Start with Docker (Recommended)

### 1. Clone the repository

```bash
git clone <repository-url>
cd Pistology-TOS
```

### 2. Start the application

```bash
docker-compose up --build
```

That's it! The application will be available at:
- **Frontend:** http://localhost
- **Backend API:** http://localhost:3001
- **Health Check:** http://localhost:3001/health

### 3. Stop the application

```bash
docker-compose down
```

### 4. Stop and remove all data

```bash
docker-compose down -v
```

## Docker Architecture

The application consists of two containerized services:

### Backend Service
- **Base Image:** node:18-alpine
- **Port:** 3001
- **Database:** SQLite (persisted in Docker volume)
- **Features:**
  - Multi-stage build for optimized image size
  - Automatic Prisma migrations on startup
  - Health checks every 30 seconds
  - Compression and caching enabled

### Frontend Service
- **Base Image:** nginx:alpine
- **Port:** 80
- **Features:**
  - Optimized production build
  - Nginx with gzip compression
  - Static asset caching
  - React Router support

### Volumes
- `backend-data`: Persists SQLite database across container restarts

### Network
- `pistology-network`: Bridge network for service communication

## Local Development (Without Docker)

### 1. Install dependencies

```bash
# Backend
cd backend
npm install

# Frontend
cd ../frontend
npm install
```

### 2. Set up environment variables

Create `backend/.env`:
```env
DATABASE_URL="file:./dev.db"
PORT=3001
```

### 3. Initialize the database

```bash
cd backend
npx prisma generate
npx prisma db push
```

### 4. Start development servers

**Terminal 1 - Backend:**
```bash
cd backend
npm run dev
```

**Terminal 2 - Frontend:**
```bash
cd frontend
npm run dev
```

The application will be available at:
- **Frontend:** http://localhost:5173
- **Backend:** http://localhost:3001

## Build for Production

### Frontend
```bash
cd frontend
npm run build
# Output: frontend/dist/
```

### Backend
```bash
cd backend
npm run build
# Output: backend/dist/
```

## API Endpoints

### Health & Status
- `GET /` - API information
- `GET /health` - Health check

### Container Management
- `GET /api/location/:containerId` - Get container location
- `POST /api/location/:containerId` - Update location
- `GET /api/history/:containerId` - Get container history
- `POST /api/history/:containerId` - Add history event
- `GET /api/damage/:containerId` - Get damage reports
- `POST /api/damage/:containerId` - Create damage report

### EDI Messages
- `GET /api/edi/:containerId` - Get EDI messages
- `POST /api/edi/:containerId` - Parse EDI message

### Customs & Inspection
- `GET /api/customs/:containerId` - Get inspection records
- `POST /api/customs/:containerId` - Create inspection

### Tasks
- `GET /api/tasks` - List all tasks
- `POST /api/tasks` - Create task
- `PUT /api/tasks/:id` - Update task
- `DELETE /api/tasks/:id` - Delete task

### Truck Appointment System
- `GET /api/appointments` - List appointments
- `POST /api/appointments` - Create appointment
- `PUT /api/appointments/:id` - Update appointment
- `DELETE /api/appointments/:id` - Delete appointment
- `GET /api/appointments/company/:name` - Get by company

## Performance Features

### Backend Optimizations
- **In-memory caching:** 5-minute cache for GET requests
- **Response compression:** Gzip compression for all responses
- **Database indexes:** Comprehensive indexing on frequently queried fields
- **Connection pooling:** Efficient database connection management

### Frontend Optimizations
- **Code splitting:** Separate chunks for vendors and routes
- **Lazy loading:** Pages load on-demand
- **Tree shaking:** Removes unused code
- **Minification:** Terser removes console.logs and debuggers
- **Asset caching:** 1-year cache for static assets

## Monitoring

### Health Checks
Both services include health checks accessible via:
```bash
# Backend
curl http://localhost:3001/health

# Frontend
curl http://localhost/health
```

### Docker Health Status
```bash
docker-compose ps
```

### Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
docker-compose logs -f frontend
```

## Database Management

### View SQLite database
```bash
# Access backend container
docker exec -it pistology-backend sh

# Install sqlite3 (if needed)
apk add sqlite

# Open database
sqlite3 /app/data/prod.db

# List tables
.tables

# Query example
SELECT * FROM Container;

# Exit
.exit
```

### Backup database
```bash
docker cp pistology-backend:/app/data/prod.db ./backup.db
```

### Restore database
```bash
docker cp ./backup.db pistology-backend:/app/data/prod.db
docker-compose restart backend
```

## Troubleshooting

### Port already in use
```bash
# Change ports in docker-compose.yml
# Frontend: "8080:80" instead of "80:80"
# Backend: "3002:3001" instead of "3001:3001"
```

### Database migration issues
```bash
# Reset database
docker-compose down -v
docker-compose up --build
```

### Container won't start
```bash
# Check logs
docker-compose logs backend
docker-compose logs frontend

# Rebuild from scratch
docker-compose down
docker-compose build --no-cache
docker-compose up
```

## Technology Stack

### Frontend
- React 18.2
- TypeScript 5
- Vite 4
- React Router 6
- Tailwind CSS 3
- React Icons

### Backend
- Node.js 18
- Express.js 4
- TypeScript 5
- Prisma ORM 6
- SQLite 3
- Zod validation
- EDIFACT parser

### DevOps
- Docker
- Docker Compose
- Nginx
- Multi-stage builds
