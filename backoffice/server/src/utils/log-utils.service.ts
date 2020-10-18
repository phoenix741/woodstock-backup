import { Response } from 'express';
import { spawn } from 'child_process';

export function tailLog(file: string, res: Response): void {
  res.header('Content-Type', 'text/plain;charset=utf-8');

  const tail = spawn('tail', ['-f', '-n', '+1', file]);
  tail.stdout.on('data', (data) => res.write(data, 'utf-8'));
  tail.stderr.on('data', (data) => res.write(data, 'utf-8'));
  tail.on('exit', (code) => res.end(code));
}

export function getLog(file: string, res: Response): void {
  res.header('Content-Type', 'text/plain;charset=utf-8');
  res.sendFile(file);
}
