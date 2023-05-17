export class DhcpAddressV4 {
  /**
   * 3 first number of the IP (that represent the network)
   */
  address!: string;
  /**
   * Start of the IP to check
   * @minimum 0
   * @maximum 255
   */
  start!: number;
  /**
   * End of the IP to check
   * @minimum 0
   * @maximum 255
   */
  end!: number;
}

export interface InformationToResolve {
  /**
   * List of hostname to test.
   */
  addresses?: string[];

  /**
   * List of DHCP range to test.
   */
  dhcp?: DhcpAddressV4[];
}
