<template>
  <Dialog :open="true">
    <DialogContent class="sqdesk-ddl-view-dialog sm:max-w-190">
      <div>fixture body</div>
      <DialogFooter>
        <Button>fixture action</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import { Dialog, DialogContent, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
</script>

<style scoped>
html.sqdesk-legacy-webview [data-slot="dialog-content"].sqdesk-ddl-view-dialog[class~="max-w-sm"] {
  max-width: 47.5rem !important;
}

html.sqdesk-legacy-webview [data-slot="dialog-content"].sqdesk-ddl-view-dialog [data-slot="dialog-footer"] {
  flex-direction: row !important;
}
</style>
