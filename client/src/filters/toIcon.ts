import Vue from 'vue';

Vue.filter('toIcon', function(value: string) {
  if (value === null || value === undefined) return '';
  switch (value) {
    case 'html':
    case 'xhtml':
    case 'htm':
      return 'mdi-language-html5';
    case 'js':
    case 'jsx':
    case 'ts':
    case 'tsx':
      return 'mdi-nodejs';
    case 'json':
    case 'json5':
    case 'yaml':
      return 'mdi-json';
    case 'md':
      return 'mdi-markdown';
    case 'pdf':
      return 'mdi-file-pdf';
    case 'png':
    case 'jpg':
    case 'gif':
    case 'jpeg':
      return 'mdi-file-image';
    case 'xls':
    case 'xlsx':
    case 'ods':
      return 'mdi-file-excel';
    case 'doc':
    case 'docx':
    case 'odt':
      return 'mdi-file-word';
    case 'zip':
      return 'mdi-zip-box';
    default:
      return 'mdi-file-document-outline';
  }
});
