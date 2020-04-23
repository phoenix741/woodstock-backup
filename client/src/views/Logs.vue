<template>
  <v-tabs vertical dark>
    <v-tab v-for="item in items" :key="item" link>
      {{ item }}
    </v-tab>

    <v-tabs-items>
      <pre>
2020-04-21T12:32:57.328Z [NestFactory              ] info: Starting Nest application...
2020-04-21T12:32:57.343Z [InstanceLoader           ] info: DiscoveryModule dependencies initialized
2020-04-21T12:32:57.343Z [InstanceLoader           ] info: ConfigHostModule dependencies initialized
2020-04-21T12:32:57.343Z [InstanceLoader           ] info: ConfigModule dependencies initialized
2020-04-21T12:32:57.344Z [InstanceLoader           ] info: BullModule dependencies initialized
2020-04-21T12:32:57.344Z [InstanceLoader           ] info: ConfigModule dependencies initialized
2020-04-21T12:32:57.348Z [InstanceLoader           ] info: BullModule dependencies initialized
2020-04-21T12:32:57.349Z [InstanceLoader           ] info: AppModule dependencies initialized
2020-04-21T12:32:57.393Z [RoutesResolver           ] info: QueueController {/queue}:
2020-04-21T12:32:57.395Z [RouterExplorer           ] info: Mapped {/queue, GET} route
2020-04-21T12:32:57.395Z [RoutesResolver           ] info: BackupController {/hosts/:name/backups}:
2020-04-21T12:32:57.395Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups, GET} route
2020-04-21T12:32:57.396Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups, POST} route
2020-04-21T12:32:57.396Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups/:number, DELETE} route
2020-04-21T12:32:57.396Z [RoutesResolver           ] info: HostController {/hosts}:
2020-04-21T12:32:57.396Z [RouterExplorer           ] info: Mapped {/hosts, GET} route
2020-04-21T12:32:57.397Z [RouterExplorer           ] info: Mapped {/hosts/:name, GET} route
2020-04-21T12:32:57.397Z [RouterExplorer           ] info: Mapped {/hosts, POST} route
2020-04-21T12:32:57.397Z [RoutesResolver           ] info: ServerController {/server}:
2020-04-21T12:32:57.397Z [RouterExplorer           ] info: Mapped {/server/status, GET} route
2020-04-21T12:32:57.397Z [RoutesResolver           ] info: BackupsFilesController {/hosts/:name/backups/:number/files}:
2020-04-21T12:32:57.397Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups/:number/files, GET} route
2020-04-21T12:32:57.397Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups/:number/files/download, GET} route
2020-04-21T12:32:57.399Z [SchedulerConfigService   ] debug: SchedulerService.loadConfig: Read the file /mnt/woodstock/config/scheduler.yml
2020-04-21T12:32:57.406Z [SchedulerConfigService   ] warn: SchedulerService.loadConfig: Can't read hosts files ENOENT: no such file or directory, open '/mnt/woodstock/config/scheduler.yml'
2020-04-21T12:32:57.415Z [NestApplication          ] info: Nest application successfully started
2020-04-21T12:33:14.628Z [HostsService             ] debug: Hosts.loadHosts: Read the file /mnt/woodstock/config/hosts.yml
2020-04-21T12:33:14.636Z [HostConsumer             ] info: START: Launch the backup of the host pc-ulrich
2020-04-21T12:33:14.637Z [BackupList               ] debug: Load backup for the host pc-ulrich.
2020-04-21T12:33:14.638Z [BackupList               ] debug: Found 3 for the host pc-ulrich
2020-04-21T12:33:14.643Z [BtrfsService             ] debug: Create snapshot /mnt/woodstock/hosts/pc-ulrich/4 from /mnt/woodstock/hosts/pc-ulrich/3
2020-04-21T12:59:30.043Z [BackupList               ] debug: Load backup for the host pc-ulrich.
2020-04-21T12:59:30.057Z [BackupList               ] debug: Found 3 for the host pc-ulrich
2020-04-21T12:59:30.057Z [BackupList               ] debug: Save backup for the host pc-ulrich.
2020-04-21T12:59:30.063Z [HostConsumer             ] debug: END: Of backup of the host pc-ulrich
2020-04-21T13:00:00.046Z [SchedulerConsumer        ] info: Scheduler wakeup at 2020-04-21T12:32:57.413Z
2020-04-21T13:00:00.049Z [HostConsumer             ] info: START: Test pc-ulrich for backup
2020-04-21T13:00:00.049Z [BackupList               ] debug: Load backup for the host pc-ulrich.
2020-04-21T13:00:00.051Z [BackupList               ] debug: Found 4 for the host pc-ulrich
2020-04-21T13:00:00.051Z [HostConsumer             ] debug: Last backup for the host pc-ulrich have been made at 0.4459475 hours past (should be made after 23.916666666666668 hour)
2020-04-21T13:00:00.052Z [HostConsumer             ] debug: END: Host pc-ulrich will not be backuped
2020-04-21T16:19:47.143Z [SchedulerConsumer        ] info: Scheduler wakeup at 2020-04-21T13:00:00.041Z
2020-04-21T16:19:47.167Z [HostConsumer             ] info: START: Test pc-ulrich for backup
2020-04-21T16:19:47.167Z [BackupList               ] debug: Load backup for the host pc-ulrich.
2020-04-21T16:19:47.171Z [BackupList               ] debug: Found 4 for the host pc-ulrich
2020-04-21T16:19:47.171Z [HostConsumer             ] debug: Last backup for the host pc-ulrich have been made at 3.7757030555555557 hours past (should be made after 23.916666666666668 hour)
2020-04-21T16:19:47.171Z [HostConsumer             ] debug: END: Host pc-ulrich will not be backuped
2020-04-21T16:35:23.812Z [NestFactory              ] info: Starting Nest application...
2020-04-21T16:35:23.830Z [InstanceLoader           ] info: DiscoveryModule dependencies initialized
2020-04-21T16:35:23.830Z [InstanceLoader           ] info: ConfigHostModule dependencies initialized
2020-04-21T16:35:23.830Z [InstanceLoader           ] info: ConfigModule dependencies initialized
2020-04-21T16:35:23.830Z [InstanceLoader           ] info: BullModule dependencies initialized
2020-04-21T16:35:23.830Z [InstanceLoader           ] info: ConfigModule dependencies initialized
2020-04-21T16:35:23.834Z [InstanceLoader           ] info: BullModule dependencies initialized
2020-04-21T16:35:23.835Z [InstanceLoader           ] info: AppModule dependencies initialized
2020-04-21T16:35:23.893Z [RoutesResolver           ] info: QueueController {/queue}:
2020-04-21T16:35:23.895Z [RouterExplorer           ] info: Mapped {/queue, GET} route
2020-04-21T16:35:23.895Z [RoutesResolver           ] info: BackupController {/hosts/:name/backups}:
2020-04-21T16:35:23.896Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups, GET} route
2020-04-21T16:35:23.896Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups, POST} route
2020-04-21T16:35:23.897Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups/:number, DELETE} route
2020-04-21T16:35:23.897Z [RoutesResolver           ] info: HostController {/hosts}:
2020-04-21T16:35:23.897Z [RouterExplorer           ] info: Mapped {/hosts, GET} route
2020-04-21T16:35:23.897Z [RouterExplorer           ] info: Mapped {/hosts/:name, GET} route
2020-04-21T16:35:23.897Z [RouterExplorer           ] info: Mapped {/hosts, POST} route
2020-04-21T16:35:23.897Z [RoutesResolver           ] info: ServerController {/server}:
2020-04-21T16:35:23.898Z [RouterExplorer           ] info: Mapped {/server/status, GET} route
2020-04-21T16:35:23.898Z [RoutesResolver           ] info: BackupsFilesController {/hosts/:name/backups/:number/files}:
2020-04-21T16:35:23.898Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups/:number/files, GET} route
2020-04-21T16:35:23.898Z [RouterExplorer           ] info: Mapped {/hosts/:name/backups/:number/files/download, GET} route
2020-04-21T16:35:23.901Z [SchedulerConfigService   ] debug: SchedulerService.loadConfig: Read the file /mnt/woodstock/config/scheduler.yml
2020-04-21T16:35:23.910Z [SchedulerConfigService   ] warn: SchedulerService.loadConfig: Can't read hosts files ENOENT: no such file or directory, open '/mnt/woodstock/config/scheduler.yml'
      </pre>
    </v-tabs-items>
  </v-tabs>
</template>

<script lang="ts">
import { Component, Vue } from "vue-property-decorator";

@Component({})
export default class Logs extends Vue {
  items = ["application.log", "exception.log"];
}
</script>

<style scoped>
pre {
  height: 100%;
  overflow-x: auto;
}
</style>
