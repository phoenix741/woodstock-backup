import { InjectQueue } from '@nestjs/bullmq';
import { Injectable } from '@nestjs/common';
import { HostsStatsUsage, PoolStatistics, QueueName, StatsDiskUsage, StatsInstantService } from '@woodstock/shared';
import { Queue } from 'bullmq';
import * as promClient from 'prom-client';

interface QueueStats {
  completed: number;
  failed: number;
  delayed: number;
  active: number;
  waiting: number;
}

interface Cache {
  ts: number;
  disk: StatsDiskUsage;
  hosts: HostsStatsUsage;
  pool: PoolStatistics;
  queue: QueueStats;
}

@Injectable()
export class PrometheusService {
  private cache: Cache;

  constructor(
    @InjectQueue(QueueName.BACKUP_QUEUE) private readonly backupQueue: Queue,
    private instantStatService: StatsInstantService,
  ) {
    promClient.collectDefaultMetrics();

    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this;
    new promClient.Gauge({
      name: 'pool_total_disk_space',
      help: 'Total disk space available on the pool directory in Mo',
      async collect() {
        const { disk } = await self.getCache();
        this.set(Number(disk.size / 1024n / 1024n));
      },
    });
    new promClient.Gauge({
      name: 'pool_total_free_space',
      help: 'Total disk space free on the pool directory in Mo',
      async collect() {
        const { disk } = await self.getCache();
        this.set(Number(disk.size / 1024n / 1024n));
      },
    });
    new promClient.Gauge({
      name: 'pool_total_used_space',
      help: 'Total disk space used on the pool directory in Mo',
      async collect() {
        const { disk } = await self.getCache();
        this.set(Number(disk.used / 1024n / 1024n));
      },
    });

    new promClient.Gauge({
      name: 'host_backup_last_backup_size',
      help: 'Size of last backup in Mo',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, Number(hosts[host].lastBackupSize / 1024n / 1024n));
        }
      },
    });
    new promClient.Gauge({
      name: 'host_backup_last_backup_time',
      help: 'Time since last backup',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, hosts[host].lastBackupTime);
        }
      },
    });
    new promClient.Gauge({
      name: 'host_backup_last_backup_age',
      help: 'Time of last backup',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, hosts[host].lastBackupAge);
        }
      },
    });
    new promClient.Gauge({
      name: 'host_backup_last_backup_duration',
      help: 'Duration of last backup',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, hosts[host].lastBackupDuration);
        }
      },
    });
    new promClient.Gauge({
      name: 'host_backup_last_backup_completed',
      help: 'Is last backup completed',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, hosts[host].lastBackupComplete);
        }
      },
    });

    new promClient.Gauge({
      name: 'host_backup_longest_chain',
      help: 'Longest backup chain',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, hosts[host].longestChain);
        }
      },
    });
    new promClient.Gauge({
      name: 'host_backup_nb_chunk',
      help: 'Number of chunks for the host',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, hosts[host].nbChunk);
        }
      },
    });
    new promClient.Gauge({
      name: 'host_backup_nb_ref',
      help: 'Number of references for the host',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, hosts[host].nbRef);
        }
      },
    });
    new promClient.Gauge({
      name: 'host_backup_size',
      help: 'Pool Size of the backup in Mo',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, Number(hosts[host].size / 1024n / 1024n));
        }
      },
    });
    new promClient.Gauge({
      name: 'host_backup_compressed_size',
      help: 'Compressed pool size of the backup in Mo',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, Number(hosts[host].compressedSize / 1024n / 1024n));
        }
      },
    });

    new promClient.Gauge({
      name: 'host_backup_count',
      help: 'Number of backups for a host',
      labelNames: ['host'],
      async collect() {
        const { hosts } = await self.getCache();
        for (const host in hosts) {
          this.set({ host }, hosts[host].backupCount);
        }
      },
    });

    new promClient.Gauge({
      name: 'pool_longest_chain',
      help: 'Pool Longest backup chain',
      async collect() {
        const { pool } = await self.getCache();
        this.set(pool.longestChain);
      },
    });
    new promClient.Gauge({
      name: 'pool_nb_chunk',
      help: 'Number of chunks in the pool',
      async collect() {
        const { pool } = await self.getCache();
        this.set(pool.nbChunk);
      },
    });
    new promClient.Gauge({
      name: 'pool_nb_ref',
      help: 'Number of references in the pool',
      async collect() {
        const { pool } = await self.getCache();
        this.set(pool.nbRef);
      },
    });
    new promClient.Gauge({
      name: 'pool_size',
      help: 'Pool Size of the pool in Mo',
      async collect() {
        const { pool } = await self.getCache();
        this.set(Number(pool.size / 1024n / 1024n));
      },
    });
    new promClient.Gauge({
      name: 'pool_compressed_size',
      help: 'Compressed pool size of the pool in Mo',
      async collect() {
        const { pool } = await self.getCache();

        this.set(Number(pool.compressedSize / 1024n / 1024n));
      },
    });
    new promClient.Gauge({
      name: 'pool_unusedSize',
      help: 'Content of the pool that is not used in Mo',
      async collect() {
        const { pool } = await self.getCache();
        this.set(Number(pool.unusedSize / 1024n / 1024n));
      },
    });

    new promClient.Gauge({
      name: 'jobs_completed_total',
      help: 'Number of job completed',
      async collect() {
        const { queue } = await self.getCache();

        this.set(queue.completed);
      },
    });
    new promClient.Gauge({
      name: 'jobs_failed_total',
      help: 'Number of job failed',
      async collect() {
        const { queue } = await self.getCache();

        this.set(queue.failed);
      },
    });
    new promClient.Gauge({
      name: 'jobs_active_total',
      help: 'Number of job active',
      async collect() {
        const { queue } = await self.getCache();

        this.set(queue.active);
      },
    });
    new promClient.Gauge({
      name: 'jobs_delayed_total',
      help: 'Number of job delayed',
      async collect() {
        const { queue } = await self.getCache();

        this.set(queue.delayed);
      },
    });
    new promClient.Gauge({
      name: 'jobs_waiting_total',
      help: 'Number of job waiting',
      async collect() {
        const { queue } = await self.getCache();

        this.set(queue.waiting);
      },
    });
  }

  private async getCache(): Promise<Cache> {
    if (!this.cache || Date.now() - this.cache.ts > 1000 * 60) {
      const disk = await this.instantStatService.getSpace();
      const hosts = await this.instantStatService.getHostsStatsUsage();
      const pool = await this.instantStatService.getPoolStatsUsage();
      const { completed, active, delayed, failed, waiting } = await this.backupQueue.getJobCounts();

      this.cache = {
        ts: Date.now(),
        disk,
        hosts,
        pool,
        queue: {
          completed,
          failed,
          delayed,
          active,
          waiting,
        },
      };
    }

    return this.cache;
  }
}
