import React, { Component } from 'react';
import { connect } from 'react-redux';
import * as C from './Const'

import { withStyles } from '@material-ui/core/styles';
import Box from '@material-ui/core/Box';
import Button from '@material-ui/core/Button';
import TextField from '@material-ui/core/TextField';
import Dialog from '@material-ui/core/Dialog';
import DialogActions from '@material-ui/core/DialogActions';
import DialogContent from '@material-ui/core/DialogContent';
import DialogContentText from '@material-ui/core/DialogContentText';
import DialogTitle from '@material-ui/core/DialogTitle';
import IconButton from '@material-ui/core/IconButton';
import { DataGrid,
         GridToolbarContainer,
         GridColumnsToolbarButton,
         GridFilterToolbarButton, } from '@material-ui/data-grid';
import Typography from '@material-ui/core/Typography';
import LaunchTwoToneIcon from '@material-ui/icons/LaunchTwoTone';
import DeleteTwoToneIcon from '@material-ui/icons/DeleteTwoTone';
import AddCircleTwoToneIcon from '@material-ui/icons/AddCircleTwoTone';

import { startFollowing, 
         stopFollowing,
         setDrawerMenuSelection,
         setViewedHandle,
         setViewedHandleImg,
         setViewedFardels,
         queryGetFardelsFromHandle, } from './store/Actions';

const useStyles = theme => ({
  root: {
    display: 'flex',
    flexWrap: 'wrap',
  },
  dataGrid: {
  	height: 420,
  	width: '100%',
  }
});

class Following extends Component {

  constructor(props) {
  	super(props);
  	this.state = { dialogOpen: false };
  }

  toolbar() {
  	return (
      <GridToolbarContainer>
        <GridColumnsToolbarButton />
        <GridFilterToolbarButton />
      </GridToolbarContainer>
    );
  }

  handleDialogCloseButtonClick() {
  	this.setState({
  	  dialogOpen: false,
  	})
  }

  handleDialogFollowButtonClick() {
  	// TODO: execute follow
  	this.setState({
  		dialogOpen: false,
  	})
  }

  handleNewFollowButtonClick() {
    this.setState({
      dialogOpen: true,
    });
  }

  handleViewHandle(handle) {
  	const {
      setDrawerMenuSelection, 
  	  setViewedHandle, 
  	  setViewedHandleImg, 
  	  setViewedFardels, 
      queryGetFardelsFromHandle,
  	} = this.props;

  	if (!handle) return;

  	// TODO: execute queryGetFardelsFromHandle
  	setViewedHandle(handle);
    queryGetFardelsFromHandle(handle, 0, 10);
    setDrawerMenuSelection(C.DRAWER_MENU_VIEW_HANDLE);
  }

  render() {
    const { classes, following } = this.props;

    const rows = following.map( (f, idx) => {
      return {
        id: idx+1, 
      	handle: f,
      	view: true,
      	delete: true,
      };
    });

    const columns = [
      { 
      	field: 'handle', 
        headerName: 'Handle', 
        width: 300,
        renderCell: (params: GridCellParams) => (
          <strong>
            {"@"+params.value}
          </strong>
        ),
      },
      {
        field: 'view',
        headerName: 'View',
        width: 100,
        renderCell: (params: GridCellParams) => (
          <IconButton 
            color="primary" 
            aria-label="View handle" 
            component="span"
            onClick={() => this.handleViewHandle(params.row.handle)}>
            <LaunchTwoToneIcon />
          </IconButton>
        ),
        sortable: false
      },
      {
        field: 'delete',
        headerName: 'Delete',
        width: 100,
        renderCell: (params: GridCellParams) => (
          <IconButton 
            color="primary" 
            aria-label="Stop following" 
            component="span">
            <DeleteTwoToneIcon />
          </IconButton>
        ),
        sortable: false
      },
    ];

    return (
      <div className={classes.root}>
        <Typography variant="h6">Following</Typography>
        <div className={classes.dataGrid}>
          <DataGrid 
            rows={rows} 
            columns={columns} 
            pageSize={5} 
            hideFooterSelectedRowCount={true}
            components={{
              Toolbar: this.toolbar,
            }}
          />
        </div>
        <Box className={classes.buttonBox} display="flex" justifyContent="flex-end" p={1}>
          <Button
            variant="contained"
            color="default"
            className={classes.button}
            startIcon={<AddCircleTwoToneIcon />}
            onClick={() => this.handleNewFollowButtonClick()}
           >
             Follow new...
           </Button>
        </Box>
        <Dialog 
          open={this.state.dialogOpen} 
          onClose={() => this.handleDialogCloseButtonClick()} 
          aria-labelledby="start-following-dialog-title">
          <DialogTitle id="start-following-dialog-title">Start following</DialogTitle>
          <DialogContent>
            <DialogContentText>
              To follow someone new enter their handle in below.
            </DialogContentText>
            <TextField
              autoFocus
              margin="dense"
              id="name"
              label="Handle"
              type="text"
              fullWidth
            />
          </DialogContent>
          <DialogActions>
            <Button onClick={() => this.handleDialogCloseButtonClick()} color="primary">
              Cancel
            </Button>
            <Button onClick={() => this.handleDialogFollowButtonClick()} color="primary">
              Follow
            </Button>
          </DialogActions>
        </Dialog>
      </div>
    );
  }
}

const mapStateToProps = (state) => {
  return {
    keplrEnabled: state.keplrEnabled,
    following: state.following,
  }
};

const mapDispatchToProps = (dispatch) => {
  return ({
    queryGetFardelsFromHandle: (handle) => { dispatch(queryGetFardelsFromHandle(handle)) },
  	setDrawerMenuSelection: (drawerMenuSelection) => { dispatch(setDrawerMenuSelection(drawerMenuSelection)) },
    setViewedHandle: (handle) => { dispatch(setViewedHandle(handle)) },
    setViewedHandleImg: (img) => { dispatch(setViewedHandleImg(img)) },
    setViewedFardels: (fardels) => { dispatch(setViewedFardels(fardels)) },
    startFollowing: (handle) => { dispatch(startFollowing(handle)) },
    stopFollowing: (handle) => { dispatch(stopFollowing(handle)) },
  });
}

export default connect(mapStateToProps, mapDispatchToProps)(withStyles(useStyles)(Following));
