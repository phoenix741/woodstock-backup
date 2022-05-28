import { Controller, Get, Res } from '@nestjs/common';
import { ApiOkResponse } from '@nestjs/swagger';
import { Response } from 'express';
import * as client from 'prom-client';

@Controller()
export class PrometheusController {
  @ApiOkResponse({
    description: 'Returns the metrics to be used in prometheus',
  })
  @Get('/metrics')
  async metrics(@Res() res: Response) {
    res.header('Content-Type', client.register.contentType);
    res.send(await client.register.metrics());
  }
}
