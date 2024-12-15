import { Type } from 'class-transformer';
import { IsArray, IsNumber, IsString, ValidateNested } from 'class-validator';

export class Ipv4Addr {
  @IsString()
  addr: string;

  @IsString()
  netmask: string;
}

export class Ipv6Addr {
  @IsString()
  addr: string;

  @IsString()
  netmask: string;
}

export class InterfaceInformation {
  @IsString()
  name: string;

  @ValidateNested()
  @Type(() => Ipv4Addr)
  ipv4?: Ipv4Addr;

  @ValidateNested()
  @Type(() => Ipv6Addr)
  ipv6?: Ipv6Addr;
}

export class RegisterClient {
  @ValidateNested({ each: true })
  @IsArray()
  @Type(() => InterfaceInformation)
  addresses: InterfaceInformation[];

  @IsNumber()
  port: number;

  @IsString()
  version: string;
}
