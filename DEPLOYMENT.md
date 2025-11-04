# Deployment Guide - Pistology TOS

## Overview

This guide covers deploying the Pistology Terminal Operating System with Docker and SQLite backend.

## What's Been Implemented

### 1. Database Migration (PostgreSQL → SQLite)
- ✅ Updated Prisma schema to use SQLite
- ✅ Configured file-based database storage
- ✅ Added comprehensive database indexes for performance
- ✅ Updated environment configuration

### 2. Docker Configuration
- ✅ Multi-stage Dockerfile for backend (Node.js)
- ✅ Optimized Dockerfile for frontend (Nginx)
- ✅ Docker Compose orchestration
- ✅ Health checks for both services
- ✅ Volume persistence for SQLite database
- ✅ Network isolation

### 3. Performance Optimizations

#### Backend
- ✅ In-memory caching middleware (5-minute TTL)
- ✅ Automatic cache cleanup
- ✅ Response compression (gzip)
- ✅ Database indexes on all frequently queried fields:
  - Container IDs
  - Status fields
  - Timestamps
  - Foreign keys
  - Search fields (company names, license plates, etc.)

#### Frontend
- ✅ Code splitting by vendor and routes
- ✅ Lazy loading with React.lazy()
- ✅ Production build optimization
- ✅ Terser minification (removes console.logs)
- ✅ Nginx with gzip compression
- ✅ Static asset caching (1 year)

#### Security
- ✅ Security headers (X-Frame-Options, X-Content-Type-Options, X-XSS-Protection)
- ✅ CORS configuration
- ✅ Non-root user in containers

## Deployment Steps

### Prerequisites
- Docker Engine 20.10+
- Docker Compose 2.0+
- Minimum 2GB RAM
- 10GB disk space

### Step 1: Clone and Configure

```bash
git clone <repository-url>
cd Pistology-TOS
```

### Step 2: Deploy with Docker

```bash
# Build and start all services
docker-compose up --build -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f
```

### Step 3: Verify Deployment

```bash
# Check backend health
curl http://localhost:3001/health

# Check frontend
curl http://localhost/health

# Test API
curl http://localhost:3001/api/tasks
```

### Step 4: Access Application

- Frontend: http://localhost
- Backend API: http://localhost:3001
- API Documentation: See README.md

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Docker Host                          │
│                                                         │
│  ┌──────────────────────┐    ┌─────────────────────┐  │
│  │   Frontend Container │    │  Backend Container   │  │
│  │   (Nginx + React)    │◄──►│  (Node.js/Express)  │  │
│  │   Port: 80           │    │  Port: 3001          │  │
│  └──────────────────────┘    └─────────────────────┘  │
│                                        │                │
│                                        ▼                │
│                              ┌──────────────────┐      │
│                              │  SQLite Database │      │
│                              │  (Docker Volume) │      │
│                              └──────────────────┘      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Performance Benchmarks

### Expected Performance Improvements

1. **Initial Load Time**: 40-60% faster due to code splitting
2. **API Response Time**: 50-80% faster for cached requests
3. **Database Queries**: 2-10x faster with proper indexes
4. **Bundle Size**: ~30% smaller with tree shaking and minification

### Cache Statistics

Monitor cache performance:
```javascript
// Backend logs show cache hits/misses
// Example output:
// [Cache HIT] /api/location/CONT123
// [Cache MISS] /api/tasks
// [Cache] Auto-cleaned 15 expired entries
```

## Database Indexes

The following indexes have been added for optimal query performance:

### Core Models
- **Container**: containerId (unique)
- **Task**: containerId, status, assignee, createdAt
- **TruckAppointment**: containerId, truckingCompany, status, appointmentTime, licensePlate
- **EdiMessage**: containerId, messageType, createdAt
- **CustomsInspection**: containerId, status, inspectedBy, createdAt
- **DamageReport**: containerId, reportedBy, createdAt
- **History**: containerId, createdAt, event

### EDI Models
- **BaplieContainer**: baplieMessageId, containerId
- **CoarriMovement**: coarriMessageId, containerId, movementType
- **CodecoMovement**: codecoMessageId, containerId, movementType, truckLicensePlate

## Monitoring

### Health Checks

Both services have built-in health checks:

**Backend** (`/health`):
```json
{
  "status": "healthy",
  "uptime": 3600.5,
  "timestamp": "2025-01-15T10:30:00.000Z"
}
```

**Frontend** (`/health`):
```
healthy
```

### Docker Monitoring

```bash
# Container status
docker-compose ps

# Resource usage
docker stats

# Logs
docker-compose logs -f backend
docker-compose logs -f frontend

# Health status
docker inspect pistology-backend --format='{{.State.Health.Status}}'
docker inspect pistology-frontend --format='{{.State.Health.Status}}'
```

## Scaling

### Horizontal Scaling

To run multiple backend instances:

```yaml
# docker-compose.yml
services:
  backend:
    scale: 3
    # ... rest of config
```

**Note**: Consider using Redis for distributed caching when scaling horizontally.

### Resource Limits

Add resource constraints:

```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

## Backup & Restore

### Automated Backup Script

```bash
#!/bin/bash
# backup.sh
BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR
docker cp pistology-backend:/app/data/prod.db "$BACKUP_DIR/backup_$TIMESTAMP.db"
echo "Backup created: backup_$TIMESTAMP.db"

# Keep only last 7 days
find $BACKUP_DIR -name "backup_*.db" -mtime +7 -delete
```

### Restore from Backup

```bash
#!/bin/bash
# restore.sh
BACKUP_FILE=$1

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: ./restore.sh <backup_file>"
    exit 1
fi

docker-compose stop backend
docker cp "$BACKUP_FILE" pistology-backend:/app/data/prod.db
docker-compose start backend
echo "Database restored from $BACKUP_FILE"
```

## Troubleshooting

### Common Issues

1. **Port Conflicts**
   ```bash
   # Check what's using port 80
   sudo lsof -i :80

   # Change port in docker-compose.yml
   ports:
     - "8080:80"  # Use port 8080 instead
   ```

2. **Database Lock**
   ```bash
   # Stop all services
   docker-compose down

   # Remove database lock
   docker volume rm pistology_backend-data

   # Restart
   docker-compose up --build
   ```

3. **Memory Issues**
   ```bash
   # Check memory usage
   docker stats

   # Increase Docker memory limit
   # Docker Desktop: Settings > Resources > Memory
   ```

4. **Build Failures**
   ```bash
   # Clean build
   docker-compose down
   docker system prune -a --volumes
   docker-compose build --no-cache
   docker-compose up
   ```

### Debug Mode

Enable debug logging:

```bash
# Backend debug logs
docker-compose exec backend sh
export DEBUG=*
npm run dev

# Frontend with source maps
# Update vite.config.ts: sourcemap: true
```

## Production Checklist

- [ ] Update all secrets and API keys
- [ ] Configure HTTPS/SSL certificates
- [ ] Set up automated backups
- [ ] Configure monitoring (Prometheus/Grafana)
- [ ] Set up logging aggregation
- [ ] Configure firewall rules
- [ ] Enable rate limiting
- [ ] Set up CI/CD pipeline
- [ ] Configure domain name and DNS
- [ ] Test disaster recovery procedure
- [ ] Document runbook for operations team

## Environment Variables

### Backend (.env)
```env
DATABASE_URL="file:/app/data/prod.db"
PORT=3001
NODE_ENV=production
LOG_LEVEL=info
CACHE_TTL=300000
```

### Frontend (build-time)
```env
VITE_API_URL=http://localhost:3001
```

## Security Considerations

1. **Network**: Use Docker networks to isolate services
2. **Secrets**: Never commit .env files to git
3. **Updates**: Regularly update base images
4. **Scanning**: Use `docker scan` to check for vulnerabilities
5. **User**: Containers run as non-root users

## Maintenance

### Regular Tasks

**Daily**:
- Monitor health checks
- Review error logs
- Check disk usage

**Weekly**:
- Review performance metrics
- Test backups
- Update dependencies (npm audit)

**Monthly**:
- Update base Docker images
- Security vulnerability scan
- Performance optimization review

### Updates

```bash
# Update dependencies
cd backend && npm update
cd ../frontend && npm update

# Rebuild containers
docker-compose build --no-cache
docker-compose up -d

# Verify
docker-compose ps
curl http://localhost:3001/health
```

## Support

For issues or questions:
1. Check logs: `docker-compose logs -f`
2. Review documentation: README.md
3. Check GitHub issues
4. Contact development team

## License

See LICENSE file for details.
